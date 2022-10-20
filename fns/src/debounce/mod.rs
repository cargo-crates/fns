use std::sync::{mpsc, Arc, Mutex};
use std::pin::Pin;
use std::time::{/* SystemTime, UNIX_EPOCH, */Duration};

pub fn debounce<F, T>(closure: F, delay: Duration) -> Debounce<T> where F: Fn(T) -> () + Send + Sync + 'static, T: Send + Sync + 'static{
  let (sender, receiver) = mpsc::channel();
  let sender = Arc::new(Mutex::new(sender));
  let debounce_config = Arc::new(Mutex::new(DebounceConfig {
    // thread_id: None,
    closure: Box::pin(closure),
    delay,
    param: None,
  }));

  let dup_debounce_config = debounce_config.clone();
  let debounce = Debounce {
    sender: Some(sender),
    // thread: Some(std::thread::spawn(move || {
    //   // let _current_duration = SystemTime::now().duration_since(UNIX_EPOCH).expect("Time went backwards");
    //   loop {
    //     let message = receiver.recv();
    //     match message {
    //       Ok(param) => {
    //         let debounce_config = dup_debounce_config.clone();
    //         let dup_debounce_config = debounce_config.clone();
    //         let thread_handle = std::thread::spawn(move || {
    //           let thread_id = std::thread::current().id();
    //           let delay = dup_debounce_config.lock().unwrap().delay;
    //           std::thread::sleep(delay);
    //           let dup_debounce_config = dup_debounce_config.lock().unwrap();
    //           if let Some(config_thread_id) = &dup_debounce_config.thread_id {
    //             if &thread_id == config_thread_id {
    //               (*dup_debounce_config.closure)(param);
    //             }
    //           }
    //         });
    //         let mut debounce_config = debounce_config.lock().unwrap();
    //         if debounce_config.thread_id.is_some() {
    //           debounce_config.thread_id = Some(thread_handle.thread().id());
    //         }
    //       },
    //       Err(_) => {
    //         break;
    //       }
    //     }
    //   }
    // })),
    thread: Some(std::thread::spawn(move || {
      loop {
        if dup_debounce_config.lock().unwrap().param.is_none() {
          let message = receiver.recv();
          match message {
            Ok(_message) => (),
            Err(_) => {
              break;
            }
          }
        } else {
          let message = receiver.recv_timeout(dup_debounce_config.lock().unwrap().delay);
          match message {
            Ok(_message) => (),
            Err(err) => {
              match err {
                mpsc::RecvTimeoutError::Timeout => {
                  let mut dup_debounce_config = dup_debounce_config.lock().unwrap();
                  if let Some(param) = dup_debounce_config.param.take() {
                    (*dup_debounce_config.closure)(param);
                  }
                },
                mpsc::RecvTimeoutError::Disconnected => {
                  break;
                }
              }
            }
          }
        }
      }
    })),
    debounce_config
  };
  debounce
}


struct DebounceConfig<T> {
  closure: Pin<Box<dyn Fn(T) -> () + Send + Sync + 'static>>,
  delay: Duration,
  // thread_id: Option<std::thread::ThreadId>,
  param: Option<T>,
}
impl<T> Drop for DebounceConfig<T> {
  fn drop(&mut self) {
    log::trace!("drop DebounceConfig {:?}", format!("{:p}", self));
  }
}


#[allow(dead_code)]
pub struct Debounce<T> {
  sender: Option<Arc<Mutex<mpsc::Sender<bool>>>>,
  thread: Option<std::thread::JoinHandle<()>>,
  debounce_config: Arc<Mutex<DebounceConfig<T>>>
}
impl<T> Debounce<T>{
  pub fn call(&self, param: T) {
    // self.debounce_config.lock().unwrap().thread_id = Some(std::thread::current().id());
    self.debounce_config.lock().unwrap().param = Some(param);
    self.sender.as_ref().unwrap().lock().unwrap().send(true).unwrap();
  }
  pub fn terminate(&self) {
    // self.debounce_config.lock().unwrap().thread_id = None;
    self.debounce_config.lock().unwrap().param = None;
  }
}
impl<T> Drop for Debounce<T> {
  fn drop(&mut self) {
    self.terminate();
    log::trace!("drop Debounce {:?}", format!("{:p}", self));
  }
}


#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn it_works() {
    let effect_run_times = Arc::new(Mutex::new(0));
    let dup_effect_run_times = effect_run_times.clone();
    let debounce_fn = debounce(move |_param| {
      *dup_effect_run_times.lock().unwrap() += 1;
    }, std::time::Duration::from_millis(1));
    debounce_fn.call(1);
    debounce_fn.call(2);
    std::thread::sleep(std::time::Duration::from_millis(2));
    assert_eq!(*effect_run_times.lock().unwrap(), 1);

    debounce_fn.call(3);
    debounce_fn.terminate();
    std::thread::sleep(std::time::Duration::from_millis(2));
    assert_eq!(*effect_run_times.lock().unwrap(), 1);
  }
}
