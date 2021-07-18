use std::any::TypeId;

pub fn to_string<T: ToString>(e: T) -> String {
    e.to_string()
}

fn type_id<T: 'static>() -> TypeId {
    TypeId::of::<T>()
}
