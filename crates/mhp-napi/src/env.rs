use std::ffi::CStr;
use std::mem;

use sys;
use result::{NapiError, NapiResult};

#[derive(Clone)]
pub struct NapiEnv {
    env: sys::napi_env,
}

impl From<sys::napi_env> for NapiEnv {
    fn from(env: sys::napi_env) -> Self {
        Self { env }
    }
}

impl NapiEnv {
    pub fn handle_status(&self, status: sys::napi_status) -> NapiResult<()> {
        if status == sys::napi_status::napi_ok {
            return Ok(());
        }

        let mut error_message = String::new();

        unsafe {
            let mut extended_error_info = mem::uninitialized();
            sys::napi_get_last_error_info(self.env, &mut extended_error_info);

            let raw_error_message = (*extended_error_info).error_message;
            if raw_error_message.is_null() {
                error_message.push_str("(error message is nullptr)");
            } else {
                let c_string = CStr::from_ptr(raw_error_message);
                error_message = c_string.to_string_lossy().into_owned();
            }
        }

        let exception = self.get_pending_exception_for_status(status);

        Err(NapiError::new(status, error_message, exception))
    }

    fn get_pending_exception_for_status(
        &self,
        status: sys::napi_status,
    ) -> Option<sys::napi_value> {
        let mut is_exception_pending = true;

        if status != sys::napi_status::napi_pending_exception {
            unsafe {
                sys::napi_is_exception_pending(
                    self.env,
                    &mut is_exception_pending,
                );
            }
        }

        if !is_exception_pending {
            return None;
        }

        unsafe {
            let mut exception = mem::uninitialized();
            sys::napi_get_and_clear_last_exception(self.env, &mut exception);

            if exception.is_null() {
                None
            } else {
                Some(exception)
            }
        }
    }
}
