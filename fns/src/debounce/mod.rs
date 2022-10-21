use std::sync::{mpsc, Arc, Mutex};
use std::pin::Pin;
use std::time::{/* SystemTime, UNIX_EPOCH, */Duration};

pub fn debounce<F, T>(closure: F, delay: Duration) -> Debounce<T> where F: Fn(T) -> () + Send + Sync + 'static, T: Send + Sync + 'static{
  let (sender, receiver) = mpsc::channel();
  let sender = Arc::new(Mutex::new(sender));
  let debounce_config = Arc::new(Mutex::new(DebounceConfig {
    closure: Box::pin(closure),
    delay,
  }));

  let dup_debounce_config = debounce_config.clone();
  let debounce = Debounce {
    sender: Some(sender),
    thread: Some(std::thread::spawn(move || {
      let debounce_config = dup_debounce_config;
      let mut current_param = None; // 最后被保存为执行的参数
      loop {
        if current_param.is_none() {
          let message = receiver.recv();
          match message {
            Ok(param) => current_param = param,
            Err(_) => { break; }
          }
        } else {
            let message = receiver.recv_timeout((*debounce_config.lock().unwrap()).delay);
            match message {
              Ok(param) => current_param = param,
              Err(err) => {
                match err {
                  mpsc::RecvTimeoutError::Timeout => {
                    if let Some(param) = current_param.take() {
                      (*debounce_config.lock().unwrap().closure)(param);
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
}
impl<T> Drop for DebounceConfig<T> {
  fn drop(&mut self) {
    log::trace!("drop DebounceConfig {:?}", format!("{:p}", self));
  }
}


#[allow(dead_code)]
pub struct Debounce<T> {
  sender: Option<Arc<Mutex<mpsc::Sender<Option<T>>>>>,
  thread: Option<std::thread::JoinHandle<()>>,
  debounce_config: Arc<Mutex<DebounceConfig<T>>>
}
impl<T> Debounce<T>{
  pub fn call(&self, param: T) {
    self.sender.as_ref().unwrap().lock().unwrap().send(Some(param)).unwrap();
  }
  pub fn terminate(&self) {
    self.sender.as_ref().unwrap().lock().unwrap().send(None).unwrap();
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
    let param = Arc::new(Mutex::new(0));
    let dup_effect_run_times = effect_run_times.clone();
    let dup_param = param.clone();
    let debounce_fn = debounce(move |param| {
      *dup_effect_run_times.lock().unwrap() += 1;
      *dup_param.lock().unwrap() = param;
    }, std::time::Duration::from_millis(100));
    {
      debounce_fn.call(1);
      debounce_fn.call(2);
      std::thread::sleep(std::time::Duration::from_millis(200));
      assert_eq!(*effect_run_times.lock().unwrap(), 1);
      assert_eq!(*param.lock().unwrap(), 2);
    }

    {
      debounce_fn.call(3);
      std::thread::sleep(std::time::Duration::from_millis(200));
      assert_eq!(*effect_run_times.lock().unwrap(), 2);
      assert_eq!(*param.lock().unwrap(), 3);
    }

    {
      debounce_fn.call(4);
      debounce_fn.terminate();
      std::thread::sleep(std::time::Duration::from_millis(200));
      assert_eq!(*effect_run_times.lock().unwrap(), 2);
      assert_eq!(*param.lock().unwrap(), 3);
    }
  }
}
