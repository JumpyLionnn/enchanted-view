
#[macro_export]
macro_rules! count_tts {
    ($_a:tt $_b:tt $_c:tt $_d:tt $_e:tt
     $_f:tt $_g:tt $_h:tt $_i:tt $_j:tt
     $_k:tt $_l:tt $_m:tt $_n:tt $_o:tt
     $_p:tt $_q:tt $_r:tt $_s:tt $_t:tt
     $($tail:tt)*)
        => {20usize + count_tts!($($tail)*)};
    ($_a:tt $_b:tt $_c:tt $_d:tt $_e:tt
     $_f:tt $_g:tt $_h:tt $_i:tt $_j:tt
     $($tail:tt)*)
        => {10usize + count_tts!($($tail)*)};
    ($_a:tt $_b:tt $_c:tt $_d:tt $_e:tt
     $($tail:tt)*)
        => {5usize + count_tts!($($tail)*)};
    ($_a:tt
     $($tail:tt)*)
        => {1usize + count_tts!($($tail)*)};
    () => {0usize};
}


/// Generates 2 functions to convert from a key to value and to convert from value to key
/// The function definition return type must implement the Into<result type> trait
/// first define the functions:
/// pub fn src_to_dst(key: K) -> Option<V>
/// pub fn dst_to_src(value: V) -> Option<K>
/// pub fn get_sources() -> [V]
/// pub fn get_dsts() -> [L]
/// then define the data: 
/// {
///     key: value,
///     key: value,
///     key: value,
///     key: value
/// }
/// 
/// This macro can be used to convert between enums and strings
#[macro_export]
macro_rules! key_value_match {
    (
        $src_to_dst_vis:vis fn $src_to_dst:ident ($src_to_dst_input:ident: $src_type:ty) -> Option<$src_to_dst_ret:ty>;
        $dst_to_src_vis:vis fn $dst_to_src:ident($dst_to_src_input:ident: $dst_type: ty) -> Option<$dst_to_src_ret:ty>;
        $get_srcs_vis:vis fn $get_srcs:ident() -> [$get_srcs_ret:ty];
        $get_dsts_vis:vis fn $get_dsts:ident() -> [$get_dsts_ret:ty];
        {$($src:tt : $dst:tt),+}
    ) => {
        $src_to_dst_vis fn $src_to_dst($src_to_dst_input: $src_type) -> Option<$src_to_dst_ret> {
            match $src_to_dst_input {
                $($src => Some($dst.into()),)+
                _other => None
            }
        }

        $dst_to_src_vis fn $dst_to_src(input: $dst_type) -> Option<$dst_to_src_ret> {
            match input {
                $($dst => Some($src.into()),)+
                _other => None
            }
        }

        $get_srcs_vis fn $get_srcs() -> [$get_srcs_ret; count_tts!($($src)+)] {
            [$($src.into(),)+]
        }

        $get_dsts_vis fn $get_dsts() -> [$get_dsts_ret; count_tts!($($dst)+)] {
            [$($dst.into(),)+]
        }
    }
}