use std::collections::{HashMap, VecDeque};

use crate::storage::{StorageError, buffer_pool_manager::Page};

pub struct LruKReplacer {
    k: usize,
    current_timestamp: usize,
    node_store: HashMap<usize, FrameNode>,
}

struct FrameNode {
    history: VecDeque<usize>, // Хранит до K последних таймстемпов
}

impl LruKReplacer {
    pub fn new(k: usize) -> Self {
        Self {
            k,
            current_timestamp: 0,
            node_store: HashMap::new(),
        }
    }

    pub fn record_access(&mut self, frame_id: usize) {
        self.current_timestamp += 1;
        let node = self.node_store.entry(frame_id).or_insert(FrameNode {
            history: VecDeque::new(),
        });

        node.history.push_back(self.current_timestamp);

        // Храним только K последних обращений
        if node.history.len() > self.k {
            node.history.pop_front();
        }
    }

    /// Возвращает frame_id для выселения, игнорируя pinned-страницы
    pub fn evict(&mut self, pages: &[Page]) -> Result<usize, StorageError> {
        let mut evict_frame = None;
        let mut min_time_inf = usize::MAX; // Для страниц с историей < K
        let mut min_time_lru = usize::MAX; // Для страниц с историей == K

        for (&frame_id, node) in &self.node_store {
            // Если страница закреплена (pinned), мы не имеем права ее выселять
            if pages[frame_id].is_pinned() {
                continue;
            }

            if node.history.len() < self.k {
                // Страницы с историей < K. Ищем самый старый ПЕРВЫЙ доступ (FIFO)
                let time = *node.history.front().unwrap();
                if time < min_time_inf {
                    min_time_inf = time;
                    evict_frame = Some(frame_id);
                }
            } else if min_time_inf == usize::MAX {
                // Страницы с историей == K. Рассматриваем их, только если нет кандидатов < K
                let time = *node.history.front().unwrap();
                if time < min_time_lru {
                    min_time_lru = time;
                    evict_frame = Some(frame_id);
                }
            }
        }

        if let Some(frame_id) = evict_frame {
            self.node_store.remove(&frame_id); // Очищаем историю выселенного фрейма
        }

        evict_frame.ok_or(StorageError::NothingToEvict)
    }
}
