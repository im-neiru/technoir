///|
pub fn prepare() -> Unit {
  let f6 = factorial(6)
  let fib10 = fibonacci(12)

  @tracing.info(Some("rain"), "Wallpaper plugin prepared")
  @tracing.debug(Some("math"), "Factorial of 6: \{f6}")
  @tracing.debug(Some("math"), "Fibonacci(12): \{fib10}")
}

///|
pub fn draw() -> Unit {
  @tracing.debug(Some("rain"), "Drawing wallpaper")
}

///|
pub fn close() -> Unit {
  @tracing.info(Some("rain"), "Wallpaper plugin closed")
}

///|
fn factorial(n : Int32) -> Int32 {
  if n <= 1 {
    1
  } else {
    n * factorial(n - 1)
  }
}

///|
fn fibonacci(n : Int32) -> Int32 {
  if n <= 1 {
    return n
  }
  let mut a = 0
  let mut b = 1
  for i = 2; i <= n; i = i + 1 {
    let temp = a + b
    a = b
    b = temp
  }
  b
}
