/// Describes a single SPIR-V instruction
pub enum SPIRV {
    /// This has no semantic impact and can safely be removed from a module.
    OpNop, 
    /// Make an intermediate object whose value is undefined.
    /// 1: Result Type ID
    /// 2: Result ID
    OpUndef(i32, i32),
    //OpSizeOf(i32, i32, i32),

    OpSourceContinued ,// TODO
    OpSource(i32, i32), // TODO
    OpSourceExtension(i32), // TODO

    /// Assign a name string to another instruction’s Result <id>. This has no semantic impact and can safely be removed from a module.
    OpName(i32, String), // TODO

    OpMemberName(i32, i32, String), // TODO

    OpDecorate(i32, Decoration),
    OpMemberDecorate(i32, i32, Decoration),
    OpDecorateId(i32, Decoration),

    OpExtension(String),
    OpMemoryModel(AddressingModel, MemoryModel),
    OpEntryPoint(ExecutionModel, i32, String, Vec<i32>),
    OpExecutionMode(i32, ExecutionModel, Vec<i32>),
    OpCapability(Capability),
    OpExecutionModeId(i32, ExecutionModel, Vec<i32>),

    // type declaration instructions
    OpTypeVoid(i32),
    OpTypeBool(i32),
    /// Declare a new integer type.
    /// 1: Result Type ID
    /// 2: Number of bits in the type
    /// 3: 0 if the type is signed, 1 if unsigned
    OpTypeInt(i32, i32, bool),
    /// Declare a new floating-point type.
    /// 1: Result Type ID
    /// 2: Number of bits in the type
    OpTypeFloat(i32, i32),
    /// Declare a new vector type.
    /// 1: Result Type ID
    /// 2: Component type ID
    /// 3: Number of components
    OpTypeVector(i32, i32, i32),
    /// Declare a new matrix type.
    /// 1: Result Type ID
    /// 2: Column type ID
    /// 3: Number of columns
    OpTypeMatrix(i32, i32, i32),

    // TODO image types

    /// Declare a new array type.
    /// 1: Result Type ID
    /// 2: Element type ID
    /// 3: Number of elements
    OpTypeArray(i32, i32, i32),
    /// Declare a new runtime array type. its length is not known at compile time.
    /// 1: Result Type ID
    /// 2: Element type ID
    OpTypeRuntimeArray(i32, i32),
    /// Declare a new structure type.
    /// 1: Result Type ID
    /// 2: IDs of the member types
    OpTypeStruct(i32, Vec<i32>),
    // TODO declare opaque type

    /// Declare a new pointer type.
    /// 1: Result Type ID
    /// 2: Storage class
    /// 3: Type ID
    OpTypePointer(i32, StorageClass, i32),
    /// Declare a new function type.
    /// 1: Result ID
    /// 2: Return type ID
    /// 3: IDs of the parameter types
    OpTypeFunction(i32, i32, Vec<i32>),

    // constant creation instructions
    /// Declare a true Boolean constant.
    /// 1: Result Type ID
    /// 2: Result ID
    OpConstantTrue(i32, i32),
    /// Declare a false Boolean constant.
    /// 1: Result Type ID
    /// 2: Result ID
    OpConstantFalse(i32, i32),
    /// Declare a new integer or floating point constant.
    /// 1: Result Type ID
    /// 2: Result ID
    /// 3: Value
    OpConstant(i32, i32, Value),
    OpConstantComposite(i32, i32, Vec<i32>),
    OpConstantNull(i32, i32),
    OpSpecConstantTrue(i32, i32),
    OpSpecConstantFalse(i32, i32),
    OpSpecConstant(i32, i32, Value),
    OpSpecConstantComposite(i32, i32, Vec<i32>),
    OpSpecConstantOp(i32, i32, i32, Vec<i32>), // TODO

    // memory instructions
    /// Allocate an object in memory, resulting in a pointer to it, which can be used with OpLoad and OpStore.
    OpVariable(i32, i32, StorageClass, Option<i32>),
    /// Load through a pointer.
    /// 1: Result Type ID
    /// 2: Result ID
    /// 3: Pointer ID
    /// 4: Memory operands mask
    OpLoad(i32, i32, i32, Option<i32>),
    /// Store through a pointer.
    /// 1: Pointer ID
    /// 2: Object ID
    /// 3: Memory operands mask
    OpStore(i32, i32, Option<i32>),
    /// Copy from the memory pointed to by Source
    /// to the memory pointed to by Target.
    /// Both operands must be non-void pointers of the same type.
    /// 1: Target ID
    /// 2: Source ID
    /// 3: Memory operands mask
    /// 4: Memory operands mask 2
    OpCopyMemory(i32, i32, Option<i32>, Option<i32>),
    /// Copy from the memory pointed to by Source
    /// to the memory pointed to by Target.
    /// 1: Target ID
    /// 2: Source ID
    /// 3: Size ID
    /// 4: Memory operands mask
    /// 5: Memory operands mask 2
    OpCopyMemorySized(i32, i32, i32, Option<i32>, Option<i32>),
    
    /// Create a pointer into a composite object
    ///
    /// 1. Result Type ID
    /// 2. Result ID
    /// 3. Base ID
    /// 4. Indices
    OpAccessChain(i32, i32, i32, Vec<i32>),

    /// Result is a valid Memory Semantics which includes mask bits set for the Storage Class for the specific (non-Generic) Storage Class of Pointer. 
    /// 
    /// Pointer must point to Generic Storage Class.
    ///
    /// Result Type must be an OpTypeInt with 32-bit Width and 0 Signedness.
    OpGenericPtrMemSemantics(i32, i32, i32), // TODO
    
    /// Result is true if Operand 1 and Operand 2 have the same value. Result is false if Operand 1 and Operand 2 have different values.
    /// 
    /// Result Type must be a Boolean type scalar.
    ///
    /// The types of Operand 1 and Operand 2 must be OpTypePointer of the same type.
    OpPtrEqual(i32, i32, i32),
    OpPtrNotEqual(i32, i32, i32),

    // function instructions
    /// Declare a new function.
    /// 1. Result Type ID
    /// 2. Result ID
    /// 3. Function Control
    /// 4. Function Type ID
    OpFunction(i32, i32, FunctionControl, i32),
    /// Declare a function parameter.
    /// 
    /// This instruction must immediately follow an OpFunction or OpFunctionParameter instruction. 
    /// The order of contiguous OpFunctionParameter instructions is the same order arguments are 
    /// listed in an OpFunctionCall instruction to this function
    /// 
    /// 1. Result Type ID
    /// 2. Result ID
    OpFunctionParameter(i32, i32),
    /// Declare the end of a function.
    OpFunctionEnd,

    /// Call a function.
    /// 
    /// Result Type is the type of the return value of the function. It must be the same as the Return Type operand of the Function Type operand of the Function operand.
    /// 
    /// Function is an OpFunction instruction. This could be a forward reference.
    /// 
    /// 1. Result Type ID
    /// 2. Result ID
    /// 3. Function ID
    /// 4. Arguments
    OpFunctionCall(i32, i32, i32, Vec<i32>),

    // conversion instructions
    /// Convert value numerically from floating point to unsigned integer, with round toward 0.0.
    /// 1. Result Type ID
    /// 2. Result ID
    /// 3. Floating-point value ID
    OpConvertFToU(i32, i32, i32), 
    /// Convert value numerically from floating point to signed integer, with round toward 0.0.
    /// 1. Result Type ID
    /// 2. Result ID
    /// 3. Floating-point value ID
    OpConvertFToS(i32, i32, i32),
    /// Convert value numerically from signed integer to floating point.
    /// 1. Result Type ID
    /// 2. Result ID
    /// 3. Signed integer value ID
    OpConvertSToF(i32, i32, i32),
    /// Convert value numerically from unsigned integer to floating point.
    /// 1. Result Type ID
    /// 2. Result ID
    /// 3. Unsigned integer value ID
    OpConvertUToF(i32, i32, i32),
    /// Convert value numerically from signed integer to unsigned integer.
    /// 1. Result Type ID
    /// 2. Result ID
    /// 3. Signed integer value ID
    OpUConvert(i32, i32, i32), 
    /// Convert value numerically from unsigned integer to signed integer.
    /// 1. Result Type ID
    /// 2. Result ID
    /// 3. Unsigned integer value ID
    OpSConvert(i32, i32, i32),
    /// Convert value numerically from one floating-point type to another width.
    /// 1. Result Type ID
    /// 2. Result ID
    /// 3. Floating-point value ID
    OpFConvert(i32, i32, i32),

    /// Convert a pointer’s Storage Class to Generic.
    ///
    /// Result Type must be an OpTypePointer. Its Storage Class must be Generic.
    ///
    /// Pointer must point to the Workgroup, CrossWorkgroup, or Function Storage Class.
    OpPtrCastToGeneric(i32, i32, i32), // TODO
    /// Convert a pointer’s Storage Class to a non-Generic class.
    ///
    /// Result Type must be an OpTypePointer. Its Storage Class must be Workgroup, CrossWorkgroup, or Function.
    ///
    /// Pointer must point to the Generic Storage Class.
    OpGenericCastToPtr(i32, i32, i32), // TODO

    /// Bit pattern-preserving type conversion.
    /// 1. Result Type ID
    /// 2. Result ID
    /// 3. Operand ID
    OpBitcast(i32, i32, i32),

    // composite instructions
    /// Extract a single, dynamically selected, component of a vector.
    /// 1. Result Type ID
    /// 2. Result ID
    /// 3. Vector ID
    /// 4. Index ID
    OpVectorExtractDynamic(i32, i32, i32, i32),
    /// Make a copy of a vector, with a single, variably selected, component modified.
    /// 1. Result Type ID
    /// 2. Result ID
    /// 3. Vector ID
    /// 4. Component ID
    /// 5. Index ID
    OpVectorInsertDynamic(i32, i32, i32, i32, i32),
    // TODO: OpVectorShuffle

    /// Construct a new composite object from a set of constituent objects.
    /// 1. Result Type ID
    /// 2. Result ID
    /// 3. Constituent objects
    OpCompositeConstruct(i32, i32, Vec<i32>),
    /// Extract a single component from a composite object.
    /// 1. Result Type ID
    /// 2. Result ID
    /// 3. Composite object ID
    /// 4. Indices
    OpCompositeExtract(i32, i32, i32, Vec<i32>),
    /// Make a copy of a composite object, with a single, variably selected, component modified.
    /// 1. Result Type ID
    /// 2. Result ID
    /// 3. Object ID
    /// 4. Composite ID
    /// 5. Indices
    OpCompositeInsert(i32, i32, i32, i32, Vec<i32>),

    /// Make a copy of Operand. There are no pointer dereferences involved.
    /// 1. Result Type ID
    /// 2. Result ID
    /// 3. Operand ID
    OpCopyObject(i32, i32, i32),

    // TODO: OpTranspose
    // TODO: OpCopyLogical

    // Arithmetic instructions
    /// Signed-integer subtract of Operand from 0
    /// a.k.a. negation
    /// 1. Result Type ID
    /// 2. Result ID
    /// 3. Operand ID
    OpSNegate(i32, i32, i32),
    OpFNegate(i32, i32, i32),
    /// Integer addition
    /// 1. Result Type ID
    /// 2. Result ID
    /// 3. Operand 1 ID
    /// 4. Operand 2 ID
    OpIAdd(i32, i32, i32, i32),
    OpFAdd(i32, i32, i32, i32),
    /// Integer subtraction
    /// 1. Result Type ID
    /// 2. Result ID
    /// 3. Operand 1 ID
    /// 4. Operand 2 ID
    OpISub(i32, i32, i32, i32),
    OpFSub(i32, i32, i32, i32),
    /// Integer multiplication
    /// 1. Result Type ID
    /// 2. Result ID
    /// 3. Operand 1 ID
    /// 4. Operand 2 ID
    OpIMul(i32, i32, i32, i32),
    OpFMul(i32, i32, i32, i32),
    /// Unsigned-integer division of Operand 1 divided by Operand 2.
    ///
    /// Result Type must be a scalar or vector of integer type, whose Signedness operand is 0. 
    /// 1. Result Type ID
    /// 2. Result ID
    /// 3. Operand 1 ID
    /// 4. Operand 2 ID
    OpUDiv(i32, i32, i32, i32),
    OpSDiv(i32, i32, i32, i32),
    OpFDiv(i32, i32, i32, i32),
    /// Unsigned-integer modulo of Operand 1 modulo Operand 2.
    /// 1. Result Type ID
    /// 2. Result ID
    /// 3. Operand 1 ID
    /// 4. Operand 2 ID
    OpUMod(i32, i32, i32, i32),
    /// Signed-integer remainder of Operand 1 `rem` Operand 2.
    OpSRem(i32, i32, i32, i32),
    /// Signed-integer modulo of Operand 1 modulo Operand 2.
    OpSMod(i32, i32, i32, i32),
    /// Floating-point remainder of Operand 1 `rem` Operand 2.
    OpFRem(i32, i32, i32, i32),
    OpFMod(i32, i32, i32, i32),

    OpVectorTimesScalar(i32, i32, i32, i32), // todo
    // TODO Matrix instructions
    // TODO Vector instructions

    OpIAddCarry(i32, i32, i32, i32),
    OpISubBorrow(i32, i32, i32, i32),
    OpUMulExtended(i32, i32, i32, i32),
    OpSMulExtended(i32, i32, i32, i32),
    // TODO: Dot Product instructions

    // Bit instructions
    OpShiftRightLogical(i32, i32, i32, i32),
    OpShiftRightArithmetic(i32, i32, i32, i32),
    OpShiftLeftLogical(i32, i32, i32, i32),
    OpBitwiseOr(i32, i32, i32, i32),
    OpBitwiseXor(i32, i32, i32, i32),
    OpBitwiseAnd(i32, i32, i32, i32),
    OpNot(i32, i32, i32),
    OpBitReverse(i32, i32, i32),
    OpBitCount(i32, i32, i32),

    // Relational and Logical instructions
    OpAny(i32, i32, i32),
    OpAll(i32, i32, i32),
    OpIsNan(i32, i32, i32),
    OpIsInf(i32, i32, i32),
    OpIsFinite(i32, i32, i32),
    OpIsNormal(i32, i32, i32),
    OpSignBitSet(i32, i32, i32),
    OpLessOrGreater(i32, i32, i32, i32),
    OpOrdered(i32, i32, i32),
    OpUnordered(i32, i32, i32),
    OpLogicalEqual(i32, i32, i32, i32),
    OpLogicalNotEqual(i32, i32, i32, i32),
    OpLogicalOr(i32, i32, i32, i32),
    OpLogicalAnd(i32, i32, i32, i32),
    OpLogicalNot(i32, i32, i32),
    OpSelect(i32, i32, i32, i32, i32),
    OpIEqual(i32, i32, i32, i32),
    OpINotEqual(i32, i32, i32, i32),
    OpUGreaterThan(i32, i32, i32, i32),
    OpSGreaterThan(i32, i32, i32, i32),
    OpUGreaterThanEqual(i32, i32, i32, i32),
    OpSGreaterThanEqual(i32, i32, i32, i32),
    OpULessThan(i32, i32, i32, i32),
    OpSLessThan(i32, i32, i32, i32),
    OpULessThanEqual(i32, i32, i32, i32),
    OpSLessThanEqual(i32, i32, i32, i32),
    OpFOrdEqual(i32, i32, i32, i32),
    OpFUnordEqual(i32, i32, i32, i32),
    OpFOrdNotEqual(i32, i32, i32, i32),
    OpFUnordNotEqual(i32, i32, i32, i32),
    OpFOrdLessThan(i32, i32, i32, i32),
    OpFUnordLessThan(i32, i32, i32, i32),
    OpFOrdGreaterThan(i32, i32, i32, i32),
    OpFUnordGreaterThan(i32, i32, i32, i32),
    OpFOrdLessThanEqual(i32, i32, i32, i32),
    OpFUnordLessThanEqual(i32, i32, i32, i32),
    OpFOrdGreaterThanEqual(i32, i32, i32, i32),
    OpFUnordGreaterThanEqual(i32, i32, i32, i32),

    // Control flow instructions
    /// 
    /// The SSA phi function.
    ///
    /// The result is selected based on control flow: If control reached the current block from Parent i, Result Id gets the value that Variable i had at the end of Parent i.
    ///
    /// Result Type can be any type except OpTypeVoid.
    ///
    /// Operands are a sequence of pairs: (Variable 1, Parent 1 block), (Variable 2, Parent 2 block), …​ 
    /// Each Parent i block is the label of an immediate predecessor in the CFG of the current block. 
    /// There must be exactly one Parent i for each parent block of the current block in the CFG. 
    /// If Parent i is reachable in the CFG and Variable i is defined in a block, that defining block 
    /// must dominate Parent i. All Variables must have a type matching Result Type.
    ///
    /// Within a block, this instruction must appear before all non-OpPhi instructions 
    /// (except for OpLine and OpNoLine, which can be mixed with OpPhi).
    OpPhi(i32, i32, Vec<(i32, i32)>),

    /// Declare a structured loop.
    /// 
    /// This instruction must immediately precede either an OpBranch or OpBranchConditional instruction. That is, it must be the second-to-last instruction in its block.
    OpLoopMerge(i32, i32, Vec<LoopControl>),
    /// Declare a structured selection.
    OpSelectionMerge(i32, i32, SelectionControl),
    /// The label instruction of a block.
    ///
    /// References to a block are through the Result <id> of its label.
    OpLabel(i32),
    /// Branch unconditionally to target label.
    OpBranch(i32),
    /// Branch conditionally based on the value of Condition.
    /// 1. Condition ID
    /// 2. True Label ID
    /// 3. False Label ID
    /// 4. Branch weights
    OpBranchConditional(i32, i32, i32, Option<(i32, i32)>),
    /// Multi-way branch to one of the operand label <id>.
    ///
    /// Selector must have a type of OpTypeInt. Selector is compared for equality to the Target literals.
    ///
    /// Default must be the <id> of a label. If Selector does not equal any of the Target literals, control flow branches to the Default label <id>.
    ///
    /// Target must be alternating scalar integer literals and the <id> of a label. If Selector equals a literal, control flow branches to the following label <id>. It is invalid for any two literal to be equal to each other. If Selector does not equal any literal, control flow branches to the Default label <id>. Each literal is interpreted with the type of Selector: The bit width of Selector’s type is the width of each literal’s type. If this width is not a multiple of 32-bits and the OpTypeInt Signedness is set to 1, the literal values are interpreted as being sign extended.
    ///
    /// This instruction must be the last instruction in a block.
    OpSwitch(i32, i32, i32, Vec<(i32, i32)>),
    /// Return from a function with void.
    OpReturn,
    /// Return a value from a function.
    OpReturnValue(i32),
    /// Unreachable code.
    /// This instruction must be the last instruction in a block.
    OpUnreachable,

    // TODO: lifetimes

    // atomic instructions
    /// Atomically load through a pointer.
    /// 1. Result Type ID
    /// 2. Result ID
    /// 3. Pointer ID
    /// 4. Memory scope id
    /// 5. Memory semantics id
    OpAtomicLoad(i32, i32, i32, i32, i32), // TODO: memory semantics
    /// Atomically store through a pointer.
    OpAtomicStore(i32, i32, i32, i32, i32), // TODO: memory semantics
    /// Atomically exchange.
    /// Perform the following steps atomically with respect to any other atomic accesses within Scope to the same location:
    /// 1. load through Pointer to get an Original Value,
    /// 2. get a New Value from copying Value, and
    /// 3. store the New Value back through Pointer.
    OpAtomicExchange(i32, i32, i32, i32, i32, i32), // TODO: memory semantics
    // TODO: OpAtomicCompareExchange
    // TODO: OpAtomicCompareExchangeWeak
    // TODO: OpAtomicIIncrement
    // TODO: OpAtomicIDecrement
    // TODO: OpAtomicIAdd
    // TODO: OpAtomicISub
    // TODO: OpAtomicSMin
    // TODO: OpAtomicUMin
    // and all others

    // Barrier instructions
    // TODO: OpControlBarrier
    // and others

    // missing: Group and subgroup instructions
    // missing: Device-side enqueue instructions
    // missing: pipe instructions
}
pub enum Decoration {
    RelaxedPrecision = 0,
    SpecId = 1,
    Block = 2,
    BufferBlock = 3,
    RowMajor = 4,
    ColMajor = 5,
    ArrayStride = 6,
    MatrixStride = 7,
    GLSLShared = 8,
    GLSLPacked = 9,
    CPacked = 10,
    BuiltIn = 11,
    NoPerspective = 13,
    Flat = 14,
    Patch = 15,
    Centroid = 16,
    Sample = 17,
    Invariant = 18,
    Restrict = 19,
    Aliased = 20,
    Volatile = 21,
    Constant = 22,
    Coherent = 23,
    NonWritable = 24,
    NonReadable = 25,
    Uniform = 26,
    UniformId = 27,
    SaturatedConversion = 28,
    Stream = 29,
    Location = 30,
    Component = 31,
    Index = 32,
    Binding = 33,
    DescriptorSet = 34,
    Offset = 35,
    XfbBuffer = 36,
    XfbStride = 37, 
    FuncParamAttr = 38,
    FPRoundingMode = 39,
    FPFastMathMode = 40,
    LinkageAttributes = 41,
    NoContraction = 42,
    InputAttachmentIndex = 43,
    Alignment = 44,
    MaxByteOffset = 45,
    AlignmentId = 46,
    MaxByteOffsetId = 47,
    NoSignedWrap = 4469,
    NoUnsignedWrap = 4470,
}

pub enum SelectionControl {
    None = 0,
    Flatten = 1,
    DontFlatten = 2,
}
pub enum LoopControl {
    None = 0,
    Unroll = 1,
    DontUnroll = 2,
    DependencyInfinite = 4,
    DependencyLength = 8,
    MinIterations = 16,
    MaxIterations = 32,
    IterationMultiple = 64,
    PeelFirst = 128,
    PeelLast = 256,
    PartialCount = 512,
    InitiationIntervalINTEL = 1024,
    MaxConcurrencyINTEL = 2048,
    DependencyArrayINTEL = 4096,
    PipelineEnableINTEL = 8192,
    LoopCoalesceINTEL = 16384,
    MaxInterleavingINTEL = 32768,
    SpeculatedIterationsINTEL = 65536,
    NoFusionINTEL = 131072,
    ControlDependencyINTEL = 262144,
    VectorizedLoopINTEL = 524288,
    UnrollEnableINTEL = 1048576,
    UnrollAndJamEnableINTEL = 2097152,
    ForcePartialCountINTEL = 4194304,
    PipelineDisableINTEL = 8388608,
    LoopCoalesceWithLoopINTEL = 16777216,
    MaxInterleavingWithLoopINTEL = 33554432,
    SpeculatedIterationsWithLoopINTEL = 67108864,
    NoFusionWithLoopINTEL = 134217728,
    ControlDependencyWithLoopINTEL = 268435456,
    VectorizedLoopWithLoopINTEL = 536870912,
    UnrollEnableWithLoopINTEL = 1073741824,
    UnrollAndJamEnableWithLoopINTEL = 2147483648,
    ForcePartialCountWithLoopINTEL = 4294967296,
    PipelineDisableWithLoopINTEL = 8589934592,
}
pub enum FunctionControl {
    None = 0,
    Inline = 1,
    DontInline = 2,
    Pure = 4,
    Const = 8,
}
pub enum Value {
    Int(i32),
    Float(f32),
}
pub enum StorageClass {
    UniformConstant = 0,
    Input = 1,
    Uniform = 2,
    Output = 3,
    Workgroup = 4,
    CrossWorkgroup = 5,
    Private = 6,
    Function = 7,
    Generic = 8,
    PushConstant = 9,
    AtomicCounter = 10,
    Image = 11,
    StorageBuffer = 12,
    CallableDataNV = 5328,
    CallableDataKHR = 5328,
    CallableData = 5328,
    CallableDataNV = 5328,
    CallableDataKHR = 5328,
    CallableData = 5328,
    IncomingCallableDataNV = 5329,
    IncomingCallableDataKHR = 5329,
    IncomingCallableData = 5329,
    IncomingCallableDataNV = 5329,
    IncomingCallableDataKHR = 5329,
    IncomingCallableData = 5329,
    RayPayloadNV = 5338,
    RayPayloadKHR = 5338,
    RayPayload = 5338,
    HitAttributeNV = 5339,
    HitAttributeKHR = 5339,
    HitAttribute = 5339,
    IncomingRayPayloadNV = 5342,
    IncomingRayPayloadKHR = 5342,
    IncomingRayPayload = 5342,
    ShaderRecordBufferNV = 5343,
    ShaderRecordBufferKHR = 5343,
    ShaderRecordBuffer = 5343,
    PhysicalStorageBuffer = 5349,
    PhysicalStorageBufferEXT = 5349,
    PhysicalStorageBufferKHR = 5349,
    PhysicalStorageBufferNV = 5349,
    PhysicalStorageBufferEXT = 5349,
    PhysicalStorageBufferKHR = 5349,
    PhysicalStorageBufferNV = 5349,
    CodeSectionINTEL = 5605,
    DeviceOnlyINTEL = 5606,
    HostOnlyINTEL = 5607,
    PushConstantINTEL = 5827,
    AccelerationStructureNV = 5348,
    AccelerationStructureKHR = 5348,
    AccelerationStructure = 5348,
    RayTracingNV = 5349,
    RayTracingKHR = 5349,
    RayTracing = 5349,
    Max = 0x7fffffff,
}
pub enum Capability {
    Matrix = 0,
    Shader = 1,
    Geometry = 2,
    Tessellation = 3,
    Addresses = 4,
    Linkage = 5,
    Kernel = 6,
    Vector16 = 7,
    Float16Buffer = 8,
    Float16 = 9,
    Float64 = 10,
    Int64 = 11,
    Int64Atomics = 12,
    ImageBasic = 13,
    ImageReadWrite = 14,
    ImageMipmap = 15,
    Pipes = 17,
    Groups = 18,
    DeviceEnqueue = 19,
    LiteralSampler = 20,
    AtomicStorage = 21,
    Int16 = 22,
    TessellationPointSize = 23,
    GeometryPointSize = 24,
    ImageGatherExtended = 25,
    StorageImageMultisample = 27,
    UniformBufferArrayDynamicIndexing = 28,
    SampledImageArrayDynamicIndexing = 29,
    StorageBufferArrayDynamicIndexing = 30,
    StorageImageArrayDynamicIndexing = 31,
    ClipDistance = 32,
    CullDistance = 33,
    ImageCubeArray = 34,
    SampleRateShading = 35,
    ImageRect = 36,
    SampledRect = 37,
    GenericPointer = 38,
    Int8 = 39,
    InputAttachment = 40,
    SparseResidency = 41,
    MinLod = 42,
    Sampled1D = 43,
    Image1D = 44,
    SampledCubeArray = 45,
    SampledBuffer = 46,
    ImageBuffer = 47,
    ImageMSArray = 48,
    StorageImageExtendedFormats = 49,
    ImageQuery = 50,
    DerivativeControl = 51,
    InterpolationFunction = 52,
    TransformFeedback = 53,
    GeometryStreams = 54,
    StorageImageReadWithoutFormat = 55,
    StorageImageWriteWithoutFormat = 56,
    MultiViewport = 57,
    SubgroupBallotKHR = 4423,
    SubgroupVoteKHR = 4424,
    StorageBuffer16BitAccess = 4427,
    StorageUniformBufferBlock16 = 4428,
    StorageUniform16 = 4429,
    StoragePushConstant16 = 4430,
}

pub enum ExecutionModel {
    Vertex = 0,
    TessellationControl = 1,
    TessellationEvaluation = 2,
    Geometry = 3,
    Fragment = 4,
    GLCompute = 5,
    Kernel = 6,
    TaskNV = 5267,
    MeshNV = 5268,
    RayGenerationNV = 5313,
    IntersectionNV = 5314,
    AnyHitNV = 5315,
    ClosestHitNV = 5316,
    MissNV = 5317,
    CallableNV = 5318,
    MeshTaskNV = 5319,
    MeshNV = 5320,
    RayGenerationKHR = 1030016000,
    IntersectionKHR = 1030016001,
    AnyHitKHR = 1030016002,
    ClosestHitKHR = 1030016003,
    MissKHR = 1030016004,
    CallableKHR = 1030016005,
    TaskNV = 1030017000,
    MeshNV = 1030017001,
    Max = 0x7fffffff,
}
pub enum AddressingModel {
    Logical = 0,
    Physical32 = 1,
    Physical64 = 2,
}

pub enum MemoryModel {
    Simple = 0,
    GLSL450 = 1,
    OpenCL = 2,
    Vulkan = 3,
}