#[macro_export]
macro_rules! impl_parser {
    ($impl_target:ty, $parse_arg:ident, $parse_body:block, $self:ident, $eval_arg:ident, $eval_body:block, $eval_ret:ty) => {
        $crate::impl_parse!($impl_target, $parse_arg, $parse_body);
        $crate::impl_eval!($impl_target, $self, $eval_arg, $eval_body, $eval_ret);
    };
    ($impl_target:ty, $parse_arg:ident, $parse_body:block, $self:ident, $($eval_arg:ident: $eval_arg_type:ty),*, $eval_body:block, $eval_ret:ty) => {
        $crate::impl_parse!($impl_target, $parse_arg, $parse_body);
        $crate::impl_eval!($impl_target, $self, $($eval_arg: $eval_arg_type),*, $eval_body, $eval_ret);
    };
}

#[macro_export]
macro_rules! impl_parse {
    ($impl_target:ty, $parse_arg:ident, $parse_body:block) => {
        impl $impl_target {
            pub fn parse<'a, 'b, 'c>(
                $parse_arg: $crate::interpreter::Position<'a, 'b, $crate::Interpreter<'c>>,
            ) -> $crate::interpreter::evaluators::parsers::AstParseResult<'a, 'b, 'c, Self> $parse_body
        }
    };
}

#[macro_export]
macro_rules! impl_eval {
    ($impl_target:ty, $self:ident, $eval_arg:ident, $eval_body:block, $eval_ret:ty) => {
        impl $impl_target {
            pub fn eval(&$self, $eval_arg: $crate::interpreter::evaluators::EvalArgs) -> Result<$eval_ret, $crate::interpreter::runtime::Error> $eval_body
        }
    };
    ($impl_target:ty, $self:ident, $($eval_arg:ident: $eval_arg_type:ty),*, $eval_body:block, $eval_ret:ty) => {
        impl $impl_target {
            pub fn eval(&$self, $($eval_arg: $eval_arg_type),*) -> Result<$eval_ret, $crate::interpreter::runtime::Error> $eval_body
        }
    };
}
