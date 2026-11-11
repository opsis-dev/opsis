use mlua::serde::{DeserializeOptions, SerializeOptions};
use mlua::{Lua, LuaSerdeExt, Result, Value};
use serde::Serialize;
use serde::de::DeserializeOwned;

pub fn to_lua<T: Serialize + ?Sized>(lua: &Lua, value: &T) -> Result<Value> {
    lua.to_value_with(value, SerializeOptions::new().serialize_none_to_null(false))
}

pub fn from_lua<T: DeserializeOwned>(lua: &Lua, value: Value) -> Result<T> {
    lua.from_value_with(value, DeserializeOptions::new())
}

#[cfg(test)]
mod tests {
    use proptest::prelude::*;
    use serde_json::{Map, Value as Json};

    use super::*;

    fn json() -> impl Strategy<Value = Json> {
        let leaf = prop_oneof![
            Just(Json::Null),
            any::<bool>().prop_map(Json::Bool),
            any::<i64>().prop_map(Json::from),
            (-1.0e9f64..1.0e9).prop_map(|f| Json::from(f + 0.5)),
            ".{0,8}".prop_map(Json::String),
        ];
        leaf.prop_recursive(4, 32, 6, |inner| {
            prop_oneof![
                proptest::collection::vec(inner.clone(), 0..5).prop_map(Json::Array),
                proptest::collection::btree_map("[a-z][a-z0-9]{0,6}", inner, 0..5)
                    .prop_map(|m| Json::Object(m.into_iter().collect::<Map<_, _>>())),
            ]
        })
    }

    proptest! {
        #[test]
        fn a_json_value_goes_into_lua_and_comes_back_equal(value in json()) {
            let lua = Lua::new();
            let back: Json = from_lua(&lua, to_lua(&lua, &value).unwrap()).unwrap();
            prop_assert_eq!(back, value);
        }

        #[test]
        fn node_settings_survive_a_script_reading_and_writing_them(value in json()) {
            let lua = Lua::new();
            lua.globals().set("given", to_lua(&lua, &value).unwrap()).unwrap();
            let copied: Value = lua.load("local t = { v = given } return t.v").eval().unwrap();
            let back: Json = from_lua(&lua, copied).unwrap();
            prop_assert_eq!(back, value);
        }
    }
}
