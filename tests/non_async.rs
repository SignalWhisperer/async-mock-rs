#[async_mock::async_mock]
trait SomeNonAsyncTrait {
    fn foo(&self, x: i32) -> i32;
}

#[tokio::test]
async fn non_async_trait_ok() {
    let mut mock = MockSomeNonAsyncTrait::default();
    mock.expect_foo().once().returning(|x| x + 1);

    let result = mock.foo(3);
    assert_eq!(result, 4);
}
