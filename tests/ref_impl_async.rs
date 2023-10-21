#![feature(async_closure)]
#[cfg(test)]
use async_mock::async_mock;
use async_trait::async_trait;

#[cfg_attr(test, async_mock)]
#[async_trait]
trait SomeTrait {
    fn foo(&self, x: i32) -> i32;
}

#[cfg_attr(test, async_mock)]
#[async_trait]
trait OtherTrait {
    async fn bar(&self, some: &(impl SomeTrait + Send + Sync), x: i32) -> i32;
}

#[tokio::test]
async fn test() {
    let mut mock_some_trait = MockSomeTrait::default();
    mock_some_trait.expect_foo().once().returning(|x| x + 1);

    let mut mock_other_trait = MockOtherTrait::default();
    mock_other_trait
        .expect_bar()
        .once()
        .returning_dyn(Box::new(|some, x| some.foo(x * 2)));

    assert_eq!(7, mock_other_trait.bar(&mock_some_trait, 3).await);
}
