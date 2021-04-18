use warp::filters::BoxedFilter;
use warp::Filter;

pub fn move_object<T: Clone + Sync + Send + 'static>(obj: T) -> BoxedFilter<(T,)> {
    warp::any().map(move || obj.clone()).boxed()
}
