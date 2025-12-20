//! LLVM Type Definitions for mdhavers
//!
//! Defines the runtime value representation and type system.

use inkwell::context::Context;
use inkwell::types::{BasicTypeEnum, PointerType, StructType};

/// Value type tags - must match runtime/mdh_runtime.h MdhTag enum
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum ValueTag {
    Nil = 0,
    Bool = 1,
    Int = 2,
    Float = 3,
    String = 4,
    List = 5,
    Dict = 6,
    Function = 7,
    Class = 8,
    Instance = 9,
    Range = 10,
    Set = 11,
    Closure = 12,
    Bytes = 13,
    NativeObject = 14,
}

impl ValueTag {
    pub fn as_u8(self) -> u8 {
        self as u8
    }
}

/// LLVM types used throughout codegen
pub struct MdhTypes<'ctx> {
    /// The main MdhValue struct type: { i8 tag, i64 data }
    pub value_type: StructType<'ctx>,
    /// i8 type (for tag)
    pub i8_type: inkwell::types::IntType<'ctx>,
    /// i32 type
    pub i32_type: inkwell::types::IntType<'ctx>,
    /// i64 type (for data field)
    pub i64_type: inkwell::types::IntType<'ctx>,
    /// f64 type (for floats)
    pub f64_type: inkwell::types::FloatType<'ctx>,
    /// bool type (i1)
    pub bool_type: inkwell::types::IntType<'ctx>,
    /// void type
    pub void_type: inkwell::types::VoidType<'ctx>,
    /// char* type (for strings)
    pub str_ptr_type: PointerType<'ctx>,
}

impl<'ctx> MdhTypes<'ctx> {
    pub fn new(context: &'ctx Context) -> Self {
        let i8_type = context.i8_type();
        let i64_type = context.i64_type();

        // MdhValue: { i8 tag, i64 data }
        let value_type = context.struct_type(
            &[i8_type.into(), i64_type.into()],
            false, // not packed
        );

        MdhTypes {
            value_type,
            i8_type,
            i32_type: context.i32_type(),
            i64_type,
            f64_type: context.f64_type(),
            bool_type: context.bool_type(),
            void_type: context.void_type(),
            str_ptr_type: i8_type.ptr_type(inkwell::AddressSpace::default()),
        }
    }

    /// Get the MdhValue type as a basic type
    pub fn value_basic_type(&self) -> BasicTypeEnum<'ctx> {
        self.value_type.into()
    }
}

/// Inferred type information for optimization
#[derive(Debug, Clone, PartialEq)]
pub enum InferredType {
    /// Type is unknown, use boxed value
    Unknown,
    /// Definitely nil
    Nil,
    /// Definitely boolean
    Bool,
    /// Definitely integer
    Int,
    /// Definitely float
    Float,
    /// Definitely string
    String,
    /// Definitely a list
    List,
    /// Definitely a dict
    Dict,
    /// A user-defined function
    Function,
    /// Numeric (int or float)
    Numeric,
}

impl InferredType {
    /// Check if this type is known at compile time
    pub fn is_known(&self) -> bool {
        !matches!(self, InferredType::Unknown)
    }

    /// Get the value tag for this type, if known
    pub fn tag(&self) -> Option<ValueTag> {
        match self {
            InferredType::Nil => Some(ValueTag::Nil),
            InferredType::Bool => Some(ValueTag::Bool),
            InferredType::Int => Some(ValueTag::Int),
            InferredType::Float => Some(ValueTag::Float),
            InferredType::String => Some(ValueTag::String),
            InferredType::List => Some(ValueTag::List),
            InferredType::Dict => Some(ValueTag::Dict),
            InferredType::Function => Some(ValueTag::Function),
            _ => None,
        }
    }
}
