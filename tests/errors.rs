#[async_mock::async_mock]
#[async_trait::async_trait]
trait SomeAsyncTrait {
    async fn foo(&self, x: i32) -> i32;
}

#[tokio::test]
#[should_panic(expected = "Missing returning function for `foo`")]
async fn missing_expectation() {
    let mock = MockSomeAsyncTrait::default();
    _ = mock.foo(3).await;
}

#[tokio::test]
#[should_panic(expected = "Failed expectation for `foo`. Called 1 times but expecting 2.")]
async fn call_count_short() {
    let mut mock = MockSomeAsyncTrait::default();
    mock.expect_foo().times(2).returning(|x| x + 1);
    _ = mock.foo(3).await;
}

#[tokio::test]
#[should_panic(expected = "Failed expectation for `foo`. Called 3 times but expecting 1.")]
async fn call_count_high() {
    let mut mock = MockSomeAsyncTrait::default();
    mock.expect_foo().once().returning(|x| x + 1);
    _ = mock.foo(3).await;
    _ = mock.foo(3).await;
    _ = mock.foo(3).await;
}

#[tokio::test]
#[should_panic(expected = "Missing returning function for `foo`")]
async fn no_returning_set() {
    let mut mock = MockSomeAsyncTrait::default();
    mock.expect_foo().once();
    _ = mock.foo(3).await;
}
