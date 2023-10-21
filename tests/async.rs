#[async_mock::async_mock]
#[async_trait::async_trait]
trait SomeAsyncTrait {
    async fn foo(&self, x: i32) -> i32;
    fn bar(&self, x: i32) -> i32;
}

#[tokio::test]
async fn async_method_ok() {
    let mut mock = MockSomeAsyncTrait::new();
    mock.expect_foo().once().returning(|x| x + 1);

    assert_eq!(4, mock.foo(3).await);
}

#[tokio::test]
async fn non_async_method_ok() {
    let mut mock = MockSomeAsyncTrait::new();
    mock.expect_bar().once().returning(|x| x + 1);

    let result = mock.bar(3);
    assert_eq!(result, 4);
}
