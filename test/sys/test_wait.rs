use nix::unistd::*;
use nix::unistd::Fork::*;
use nix::sys::signal::*;
use nix::sys::wait::*;
use libc::funcs::c95::stdlib::exit;

#[test]
fn test_wait_signal() {
    match fork() {
      Ok(Child) => {
          let pid = getpid();
          assert!(pid > 0);
          kill(pid, SIGKILL).ok().expect("Error: Kill Failed");
          loop {}
      },
      Ok(Parent(_child_pid)) => {
          match wait() {
              Ok(WaitStatus::Signaled(_child_pid, SIGKILL, _dumped_core)) => {}  // success
              _ => panic!("Expected WaitStatus::Signaled")
          }
      },
      // panic, fork should never fail unless there is a serious problem with the OS
      Err(_) => panic!("Error: Fork Failed")
    }
}

#[test]
fn test_wait_exit() {
    match fork() {
      Ok(Child) => unsafe { exit(12); },
      Ok(Parent(_child_pid)) => {
          let wait_status = wait();
          assert_eq!(wait_status, Ok(WaitStatus::Exited(_child_pid, 12)));
      },
      // panic, fork should never fail unless there is a serious problem with the OS
      Err(_) => panic!("Error: Fork Failed")
    }
}
