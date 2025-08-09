// Demo code for "Functions and Callbacks in NAPI-RS" blog post https://napi.rs/blog/function-and-callbacks

use napi::bindgen_prelude::*;
use napi::threadsafe_function::{ThreadsafeFunction, ThreadsafeFunctionCallMode};
use napi_derive::napi;
use std::thread;

// ============================================================================
// Basic Function Callbacks - Synchronous
// ============================================================================

/// Demonstrates basic synchronous callback usage
/// Takes a username and a callback that processes the uppercase version
#[napi]
pub fn process_user_data(username: String, callback: Function<String, String>) -> Result<String> {
  // Transform username to uppercase
  let processed = username.to_uppercase();

  // Call the JS callback with the processed data
  let greeting = callback.call(processed)?;

  Ok(greeting)
}

// ============================================================================
// Multiple Arguments with FnArgs
// ============================================================================

/// Demonstrates passing multiple arguments to a callback using FnArgs
/// Calculates salary components and passes them to the callback
#[napi]
pub fn calculate_salary(
  base_amount: f64,
  callback: Function<FnArgs<(f64, f64, String)>, f64>,
) -> Result<f64> {
  let tax = base_amount * 0.2;
  let department = "Engineering".to_string();

  // Pass multiple arguments using FnArgs
  callback.call((base_amount, tax, department).into())
}

// ============================================================================
// Function References for Delayed Execution
// ============================================================================

/// Demonstrates using function references to call JS functions later
/// Schedules a notification callback after a delay
#[napi(ts_return_type = "Promise<void>")]
pub fn schedule_notification<'env>(
  env: &'env Env,
  delay_ms: u32,
  callback: Function<'env, String, ()>,
) -> Result<PromiseRaw<'env, ()>> {
  // Create a reference to keep the function alive
  let callback_ref = callback.create_ref()?;

  env.spawn_future_with_callback(
    async move {
      tokio::time::sleep(std::time::Duration::from_millis(delay_ms as u64)).await;
      Ok("Notification triggered!".to_string())
    },
    move |env, message| {
      // Borrow the function back from reference
      let callback = callback_ref.borrow_back(env)?;
      callback.call(message)?;
      Ok(())
    },
  )
}

// ============================================================================
// Basic ThreadsafeFunction - Cross-thread Callbacks
// ============================================================================

/// Demonstrates calling JavaScript from a background thread
/// Simulates monitoring system resources with periodic callbacks
#[napi]
pub fn monitor_system_resources(callback: ThreadsafeFunction<f64>) {
  thread::spawn(move || {
    for i in 0..5 {
      // Simulate varying CPU usage
      let cpu_usage = 40.0 + (i as f64 * 5.0);

      // Call from background thread
      callback.call(Ok(cpu_usage), ThreadsafeFunctionCallMode::NonBlocking);

      thread::sleep(std::time::Duration::from_secs(1));
    }
  });
}

// ============================================================================
// Building ThreadsafeFunction from Function
// ============================================================================

/// Demonstrates converting a Function to ThreadsafeFunction
/// Simulates a file watcher that reports file changes
#[napi]
pub fn start_file_watcher(callback: Function<FnArgs<(String, i32)>, ()>) -> Result<()> {
  // Convert to ThreadsafeFunction with queue size limit
  let tsfn = callback
    .build_threadsafe_function()
    .max_queue_size::<10>() // Limit queue to 10 items
    .build()?;

  thread::spawn(move || {
    let files = vec![
      ("config.json", 1024),
      ("data.csv", 2048),
      ("index.html", 512),
      ("styles.css", 768),
      ("script.js", 1536),
    ];

    for (filename, size) in files {
      tsfn.call(
        (filename.to_string(), size).into(),
        ThreadsafeFunctionCallMode::Blocking,
      );
      thread::sleep(std::time::Duration::from_millis(500));
    }
  });

  Ok(())
}

// ============================================================================
// Async Operations with ThreadsafeFunction
// ============================================================================

/// Demonstrates async/await with ThreadsafeFunction returning a Promise
/// Fetches user profile data asynchronously
#[napi]
pub async fn fetch_user_profile(
  user_id: u32,
  callback: ThreadsafeFunction<String, Promise<String>>,
) -> Result<String> {
  // Prepare the user ID string
  let user_key = format!("user_{}", user_id);

  // Call async and await the Promise from JavaScript
  let profile_promise = callback.call_async(Ok(user_key)).await?;
  let enhanced_profile = profile_promise.await?;

  Ok(format!("Enhanced: {}", enhanced_profile))
}

// ============================================================================
// Custom Error Handling with ThreadsafeFunction
// ============================================================================

/// Custom error type for network operations
pub struct NetworkError(String);

impl AsRef<str> for NetworkError {
  fn as_ref(&self) -> &str {
    &self.0
  }
}

impl From<Status> for NetworkError {
  fn from(_: Status) -> Self {
    NetworkError("Network failure".to_string())
  }
}

/// Demonstrates custom error handling in ThreadsafeFunction
/// Simulates a file download with progress updates and potential failure
#[napi]
pub fn download_file_with_progress(
  url: String,
  callback: ThreadsafeFunction<u32, (), u32, NetworkError>,
) {
  thread::spawn(move || {
    println!("Starting download from: {}", url);

    for progress in (0..=100).step_by(20) {
      if progress == 60 {
        // Simulate network error at 60%
        callback.call(
          Err(Error::new(
            NetworkError("Connection lost".to_string()),
            format!("Download failed at {}%", progress),
          )),
          ThreadsafeFunctionCallMode::Blocking,
        );
        return;
      }

      // Report progress
      callback.call(Ok(progress), ThreadsafeFunctionCallMode::Blocking);
      thread::sleep(std::time::Duration::from_millis(200));
    }
  });
}

// ============================================================================
// Different Call Modes: Blocking vs NonBlocking
// ============================================================================

/// Demonstrates the difference between Blocking and NonBlocking call modes
/// Processes events with different priorities
#[napi]
pub fn process_events(
  high_priority: ThreadsafeFunction<String>,
  low_priority: ThreadsafeFunction<String>,
) {
  thread::spawn(move || {
    // High priority: block to ensure delivery
    high_priority.call(
      Ok("CRITICAL: System alert - CPU temperature high!".to_string()),
      ThreadsafeFunctionCallMode::Blocking,
    );

    // Low priority: drop if queue is full
    low_priority.call(
      Ok("INFO: Regular system check completed".to_string()),
      ThreadsafeFunctionCallMode::NonBlocking,
    );

    // More events
    for i in 0..3 {
      let msg = format!("INFO: Update #{}", i + 1);
      low_priority.call(Ok(msg), ThreadsafeFunctionCallMode::NonBlocking);
      thread::sleep(std::time::Duration::from_millis(100));
    }
  });
}

// ============================================================================
// Weak References - Background Tasks
// ============================================================================

/// Demonstrates weak references that don't keep the process alive
/// Creates a background logger that won't prevent process termination
#[napi]
pub fn background_logger(callback: Function<String, ()>) -> Result<()> {
  let tsfn = callback
    .build_threadsafe_function()
    .callee_handled::<true>()
    .weak::<true>() // Weak reference - won't prevent process exit
    .build()?;

  thread::spawn(move || {
    let mut counter = 0;
    loop {
      counter += 1;
      let log_msg = format!("Background log entry #{}", counter);

      tsfn.call(Ok(log_msg), ThreadsafeFunctionCallMode::NonBlocking);

      thread::sleep(std::time::Duration::from_secs(2));
    }
  });

  Ok(())
}

// ============================================================================
// Complex Example: Real-time Data Stream
// ============================================================================

/// A more complex example simulating a real-time data stream
/// with multiple callback types and error handling
#[napi]
pub fn stream_sensor_data(
  data_callback: ThreadsafeFunction<FnArgs<(String, f64, u32)>>,
  error_callback: Function<String, ()>,
) -> Result<()> {
  // Convert error callback to threadsafe
  let error_tsfn = error_callback
    .build_threadsafe_function()
    .callee_handled::<true>()
    .build()?;

  thread::spawn(move || {
    let sensors = vec!["temperature", "humidity", "pressure"];
    let mut timestamp = 0u32;

    for _ in 0..10 {
      for sensor in sensors.iter() {
        timestamp += 1000;

        // Simulate sensor reading (simplified without rand crate)
        let value = match sensor.as_ref() {
          "temperature" => 22.5 + ((timestamp % 10) as f64),
          "humidity" => 45.0 + ((timestamp % 20) as f64),
          "pressure" => 1013.0 + ((timestamp % 50) as f64),
          _ => 0.0,
        };

        // Simulate occasional sensor errors
        if value > 55.0 && *sensor == "humidity" {
          error_tsfn.call(
            Ok(format!("Sensor {} reading out of range: {}", sensor, value)),
            ThreadsafeFunctionCallMode::NonBlocking,
          );
          continue;
        }

        // Send sensor data
        data_callback.call(
          Ok((sensor.to_string(), value, timestamp).into()),
          ThreadsafeFunctionCallMode::NonBlocking,
        );

        thread::sleep(std::time::Duration::from_millis(100));
      }
    }
  });

  Ok(())
}

// ============================================================================
// Callback with Return Value Processing
// ============================================================================

/// Demonstrates processing return values from threadsafe function callbacks
#[napi]
pub fn process_with_feedback(items: Vec<String>, processor: ThreadsafeFunction<String, String>) {
  thread::spawn(move || {
    for item in items {
      // Call with return value processing
      processor.call_with_return_value(
        Ok(item.clone()),
        ThreadsafeFunctionCallMode::Blocking,
        move |result, _env| {
          match result {
            Ok(processed) => {
              println!("Processed '{}' -> '{}'", item, processed);
            }
            Err(e) => {
              println!("Error processing '{}': {}", item, e);
            }
          }
          Ok(())
        },
      );

      thread::sleep(std::time::Duration::from_millis(200));
    }
  });
}

// ============================================================================
// Helper Functions for Testing
// ============================================================================

/// Simple sync function for testing
#[napi]
pub fn simple_callback_test(value: i32, callback: Function<i32, i32>) -> Result<i32> {
  let doubled = value * 2;
  callback.call(doubled)
}

/// Test function with optional callback
#[napi]
pub fn optional_callback_test(
  value: String,
  callback: Option<Function<String, ()>>,
) -> Result<String> {
  let result = format!("Processed: {}", value);

  if let Some(cb) = callback {
    cb.call(result.clone())?;
  }

  Ok(result)
}
