#[async_mock::async_mock]
#[async_trait::async_trait]
trait SomeTrait {
    async fn some_method(&self) -> Result<u32, &'static str>;
}

#[derive(Default)]
struct SomeStruct;

impl SomeStruct {
    async fn do_something(&self, t: &impl SomeTrait) -> Result<u32, &'static str> {
        t.some_method().await
    }
}

#[tokio::test]
async fn all_ok() {
    let mut mock = mocks::MockSomeTrait::default();
    mock.expect_some_method().once().returning(|| Ok(42));

    let sut = SomeStruct::default();
    let result = sut.do_something(&mock).await;

    assert_eq!(result, Ok(42));
}

#[tokio::test]
#[should_panic(expected = "Missing returning function for `some_method`")]
async fn missing_expectation() {
    let mock = mocks::MockSomeTrait::default();

    let sut = SomeStruct::default();
    _ = sut.do_something(&mock).await;
}

#[tokio::test]
#[should_panic(expected = "Failed expectation for `some_method`. Called 1 times but expecting 2.")]
async fn call_count_short() {
    let mut mock = mocks::MockSomeTrait::default();
    mock.expect_some_method().times(2).returning(|| Ok(42));

    let sut = SomeStruct::default();
    _ = sut.do_something(&mock).await;
}

#[tokio::test]
#[should_panic(expected = "Failed expectation for `some_method`. Called 2 times but expecting 1.")]
async fn call_count_high() {
    let mut mock = mocks::MockSomeTrait::default();
    mock.expect_some_method().once().returning(|| Ok(42));

    let sut = SomeStruct::default();
    _ = sut.do_something(&mock).await;
    _ = sut.do_something(&mock).await;
}

#[tokio::test]
#[should_panic(expected = "Missing returning function for `some_method`")]
async fn no_returning_set() {
    let mut mock = mocks::MockSomeTrait::default();
    mock.expect_some_method().once();

    let sut = SomeStruct::default();
    _ = sut.do_something(&mock).await;
}
