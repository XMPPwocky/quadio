/*pub struct ParameterMeta {
    label: String
}

macro_rules! param {
    (
        struct $struct_name:ident: $desc_name:ident {
            $(
                $field_name:ident : $param_ty:ty
            ),*
        }
    ) => {
        pub struct $struct_name<'a> {
            $(
                $field_name: &'a [$param_ty]
            ),*
        }
        pub struct $desc_name {
            $(
                $field_name: ParameterMeta
            ),*
        }
        impl std::default::Default for $desc_name {
            fn default() -> $desc_name {
                $desc_name {
                    $(
                        $field_name: ParameterMeta { label: stringify!($field_name).to_owned() }
                    ),*
                }
            }
        }
    };
}

param!(struct Foob: FoobParam {
    z: f32,
    y: u8
});
*/
