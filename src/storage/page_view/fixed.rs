use std::{backtrace::Backtrace, marker::PhantomData};

use zerocopy::{FromBytes, Immutable, IntoBytes, KnownLayout, Ref};

use crate::{storage::StorageError, storage::common::PAGE_SIZE};

/// Заголовок страницы: битовая маска занятости слотов.
/// 1 бит = слот занят, 0 бит = слот свободен.
/// PAGE_SIZE / size_of::<T>() не должно превышать HEADER_BITS.
const HEADER_BITS: usize = 512; // максимальное количество слотов
const HEADER_BYTES: usize = HEADER_BITS / 8; // 64 байта


#[derive(FromBytes, IntoBytes, Immutable, KnownLayout, Clone, Copy)]
#[repr(C)]
pub struct PageHeader {
    occupied: [u8; HEADER_BYTES],
}

impl PageHeader {
    pub fn is_occupied(&self, slot_id: u16) -> bool {
        let byte = slot_id as usize / 8;
        let bit  = slot_id as usize % 8;
        (self.occupied[byte] >> bit) & 1 == 1
    }

    pub fn set_occupied(&mut self, slot_id: u16) {
        let byte = slot_id as usize / 8;
        let bit  = slot_id as usize % 8;
        self.occupied[byte] |= 1 << bit;
    }

    pub fn set_free(&mut self, slot_id: u16) {
        let byte = slot_id as usize / 8;
        let bit  = slot_id as usize % 8;
        self.occupied[byte] &= !(1 << bit);
    }
}


/// Смещение данных слотов — после заголовка.
const DATA_OFFSET: usize = std::mem::size_of::<PageHeader>();

pub struct FixedPageView<B, T> {
    buffer: B,
    _marker: PhantomData<T>,
}

impl<'a, T> FixedPageView<&'a [u8], T> {
    pub fn new_read(buf: &'a [u8]) -> Self {
        Self { buffer: buf, _marker: PhantomData }
    }
}

impl<'a, T> FixedPageView<&'a mut [u8], T> {
    pub fn new_write(buf: &'a mut [u8]) -> Self {
        Self { buffer: buf, _marker: PhantomData }
    }
}

// ── Вспомогательные методы для заголовка ────────────────────────────────────

impl<B, T> FixedPageView<B, T>
where
    B: AsRef<[u8]>,
{
    fn header(&self) -> &PageHeader {
        PageHeader::ref_from_bytes(&self.buffer.as_ref()[..DATA_OFFSET]).unwrap()
    }
}

impl<B, T> FixedPageView<B, T>
where
    B: AsMut<[u8]>,
{
    fn header_mut(&mut self) -> &mut PageHeader {
        PageHeader::mut_from_bytes(&mut self.buffer.as_mut()[..DATA_OFFSET]).unwrap()
    }
}

// ── Чтение ──────────────────────────────────────────────────────────────────

impl<B, T> FixedPageView<B, T>
where
    B: AsRef<[u8]>,
    T: FromBytes + IntoBytes + Immutable + KnownLayout + Copy,
{
    pub const fn capacity() -> usize {
        let size = (PAGE_SIZE - DATA_OFFSET) / std::mem::size_of::<T>();
        if size > HEADER_BITS {
            return HEADER_BITS;
        }
        size
    }

    pub fn check_slot_id(slot_id: u16) -> Result<(), StorageError> {
        if slot_id as usize >= Self::capacity() {
            Err(StorageError::SlotOutOfRange(slot_id, Backtrace::capture()))
        } else {
            Ok(())
        }
    }

    fn get_slot_ref(&self, slot_id: u16) -> Ref<&[u8], T> {
        let start = DATA_OFFSET + slot_id as usize * std::mem::size_of::<T>();
        let end   = start + std::mem::size_of::<T>();
        Ref::from_bytes(&self.buffer.as_ref()[start..end]).unwrap()
    }

    pub fn get(&self, slot_id: u16) -> Result<Option<&T>, StorageError> {
        Self::check_slot_id(slot_id)?;
        if self.header().is_occupied(slot_id) {
            Ok(Some(Ref::into_ref(self.get_slot_ref(slot_id))))
        } else {
            Ok(None)
        }
    }

    pub fn has_free_slot(&self) -> bool {
        (0..Self::capacity() as u16).any(|id| !self.header().is_occupied(id))
        // let free_space = 
        // let header = self.header();
        // for byte in &header.occupied {
        //     if *byte != 0xFF { // Если не все биты заняты

        //         return true;
        //     }
        // }
        // false
    }


    pub fn print_header(&self) {
        let header = self.header();
        print!("Header: ");
        for i in 0..Self::capacity() {
            if header.is_occupied(i as u16) {
                print!("1");
            } else {
                print!("0");
            }
        }
        println!();
    }

}

// ── Запись ──────────────────────────────────────────────────────────────────

impl<B, T> FixedPageView<B, T>
where
    B: AsRef<[u8]> + AsMut<[u8]>,
    T: FromBytes + IntoBytes + Immutable + KnownLayout + Copy,
{
    fn get_slot_mut(&mut self, slot_id: u16) -> Ref<&mut [u8], T> {
        let start = DATA_OFFSET + slot_id as usize * std::mem::size_of::<T>();
        let end   = start + std::mem::size_of::<T>();
        Ref::from_bytes(&mut self.buffer.as_mut()[start..end]).unwrap()
    }

    // TODO: оптимизировать поиск свободного слота, например, с помощью битовой маски в заголовке
    pub fn insert(&mut self, record: T) -> Option<u16> {
        let slot_id = (0..Self::capacity() as u16)
            .find(|&id| !self.header().is_occupied(id))?;

        *self.get_slot_mut(slot_id) = record;
        self.header_mut().set_occupied(slot_id);
        Some(slot_id)
    }

    pub fn set(&mut self, slot_id: u16, record: T) -> Result<(), StorageError> {
        Self::check_slot_id(slot_id)?;
        *self.get_slot_mut(slot_id) = record;
        self.header_mut().set_occupied(slot_id);
        Ok(())
    }

    pub fn get_mut(&mut self, slot_id: u16) -> Result<Option<&mut T>, StorageError> {
        Self::check_slot_id(slot_id)?;
        if self.header().is_occupied(slot_id) {
            Ok(Some(Ref::into_mut(self.get_slot_mut(slot_id))))
        } else {
            Ok(None)
        }
    }

    pub fn delete(&mut self, slot_id: u16) -> Result<(), StorageError> {
        Self::check_slot_id(slot_id)?;
        self.header_mut().set_free(slot_id);
        Ok(())
    }
}