#[cfg(test)]
use async_mock::async_mock;
use async_trait::async_trait;

#[cfg_attr(test, async_mock)]
#[async_trait]
trait MyTrait {
    async fn foo(&self, x: i32) -> i32;
}

#[derive(Default)]
struct MyStruct;
impl MyStruct {
    async fn bar(&self, my_trait: &impl MyTrait, x: i32) -> i32 {
        my_trait.foo(x * 2).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn some_test() {
        let mut mock = MockMyTrait::default();
        mock.expect_foo().times(1).returning(|x| x + 1);

        let system_under_test = MyStruct::default();
        assert_eq!(7, system_under_test.bar(&mock, 3).await);
    }
}
