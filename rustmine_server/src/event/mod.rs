pub mod player_events;

use std::{any::{Any, TypeId}, collections::HashMap, pin::Pin, sync::Arc};

use tokio::sync::RwLock;

pub trait Event<R: Clone + Send + Sync + 'static>: Send + Sync + 'static {}

//
// One thing to ensure during event thingys, is that the Shared<T> values are not locked prior to the dispatch
//

pub struct EventBus {
    listeners: RwLock<
        HashMap<
            TypeId,
            Vec<(
                bool,
                Box<
                    dyn Fn(
                            Arc<RwLock<Box<dyn Any + Send + Sync>>>,
                        ) -> Pin<
                            Box<dyn Future<Output = Option<Box<dyn Any + Send + Sync>>> + Send>,
                        > + Send
                        + Sync,
                >,
            )>,
        >,
    >,
}

impl Default for EventBus {
    fn default() -> Self {
        Self { listeners: Default::default() }
    }
}

impl EventBus {
    pub async fn listen<E,R, F, Fut>(&self, lazy: bool, callback: F)
    where
    E: Event<R>,
    R: Clone + Send + Sync + 'static,
        F: Fn(Arc<E>) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Option<R>> + Send + 'static,
    {
        let mut listeners = self.listeners.write().await;
        listeners
            .entry(TypeId::of::<E>())
            .or_insert_with(Vec::new)
            .push((
                lazy,
                Box::new({
                    let callback = Arc::new(callback);
                    move |event| {
                        let callback = Arc::clone(&callback);
                        Box::pin(async move {
                            let event = event.read().await;
                            if let Some(event) = event.downcast_ref::<Arc<E>>() {
                                callback( event.clone())
                                    .await
                                    .map(|res| Box::new(res) as Box<dyn Any + Send + Sync>)
                            } else {
                                None
                            }
                        })
                    }
                }),
            ));
    }

    pub async fn dispatch<E: Event<R>,R: Clone + Send + Sync + 'static,>(&self, event: &Arc<E>) -> Option<R> {
        let listeners = self.listeners.read().await;
        let event: Arc<RwLock<Box<dyn Any + Send + Sync>>> = Arc::new(RwLock::new(Box::new(event.clone())));
    
        let mut last_result = None;
        let mut tasks = Vec::new();
    
        for (lazy, handler) in listeners.get(&TypeId::of::<E>()).into_iter().flatten() {
            let event = Arc::clone(&event);
    
            if *lazy {
                tokio::spawn(handler(event));
            } else {
                tasks.push(handler(event));
            }
        }
    
        for task in tasks {
            if let Some(result_boxed) = task.await {
                if let Ok(result) = result_boxed.downcast::<R>() {
                    last_result = Some(*result);
                }
            }
        } // Rethink this, maybe no event results? 
    
        last_result
    }
}