#[async_mock::async_mock]
#[async_trait::async_trait]
trait SomeAsyncTrait {
    async fn foo(&self, x: i32) -> i32;
    fn bar(&self, x: i32) -> i32;
}

#[derive(Default)]
struct SomeStruct;

impl SomeStruct {
    async fn foo_di(&self, t: &impl SomeAsyncTrait, x: i32) -> i32 {
        t.foo(x).await
    }

    async fn bar_di(&self, t: &impl SomeAsyncTrait, x: i32) -> i32 {
        t.bar(x)
    }
}

#[tokio::test]
async fn async_ok() {
    let mut mock = MockSomeAsyncTrait::default();
    mock.expect_foo().once().returning(|x| x + 1);

    let sut = SomeStruct::default();
    let result = sut.foo_di(&mock, 3).await;
    assert_eq!(result, 4);
}

#[tokio::test]
async fn non_async_ok() {
    let mut mock = MockSomeAsyncTrait::default();
    mock.expect_bar().once().returning(|x| x + 1);

    let sut = SomeStruct::default();
    let result = sut.bar_di(&mock, 3).await;
    assert_eq!(result, 4);
}
