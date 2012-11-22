
struct ClassFile {
    magic: u32,
    minor_version: u16,
    major_version: u16,
    constant_pool: ~[ConstantPoolInfo],
    access_flags: u16,
    this_class: u16,
    super_class: u16,
    interfaces: ~[u16],
    fields: ~[FieldInfo],
    methods: ~[MethodInfo],
    attributes: ~[AttributeInfo]
}

enum ClassAccessFlag {
    ClassAccess_Public = 0x0001,
    ClassAccess_Static = 0x0008,
    ClassAccess_Final = 0x0010,
    ClassAccess_Super = 0x0020,
    ClassAccess_Interface = 0x0200,
    ClassAccess_Abstract = 0x0400,
    ClassAccess_Synthetic = 0x1000,
    ClassAccess_Annotation = 0x2000,
    ClassAccess_Enum = 0x4000
}

enum ConstantPoolTag {
    PoolTag_Utf8 = 1,
    PoolTag_Integer = 3,
    PoolTag_Float = 4,
    PoolTag_Long = 5,
    PoolTag_Double = 6,
    PoolTag_Class = 7,
    PoolTag_String = 8,
    PoolTag_FieldRef = 9,
    PoolTag_MethodRef = 10,
    PoolTag_InterfaceMethodRef = 11,
    PoolTag_NameAndType = 12,
    PoolTag_MethodHandle = 15,
    PoolTag_MethodType = 16,
    PoolTag_InvokeDynamic = 18
}

//ref: rust #2132
fn ConstantPoolTag_from_int(ordinal: int) -> ConstantPoolTag {
    match ordinal {
        1 => PoolTag_Utf8,
        3 => PoolTag_Integer,
        4 => PoolTag_Float,
        5 => PoolTag_Long,
        6 => PoolTag_Double,
        7 => PoolTag_Class,
        8 => PoolTag_String,
        9 => PoolTag_FieldRef,
        10 => PoolTag_MethodRef,
        11 => PoolTag_InterfaceMethodRef,
        12 => PoolTag_NameAndType,
        15 => PoolTag_MethodHandle,
        16 => PoolTag_MethodType,
        18 => PoolTag_InvokeDynamic,
        _ => fail fmt!("Unrecognized ordinal %?", ordinal)
    }
}

struct ConstantPoolInfo {
    tag: ConstantPoolTag,
    info: ConstantPoolStructure
}

fn ConstantPoolInfo(reader: io::ReaderUtil) -> ConstantPoolInfo {
    let tag = ConstantPoolTag_from_int(read_u8(reader) as int);
    let inf: ConstantPoolStructure = match tag {
        PoolTag_Class => PoolStruct_Class(read_u16(reader)),
        PoolTag_FieldRef => PoolStruct_FieldRef(read_u16(reader), read_u16(reader)),
        PoolTag_MethodRef => PoolStruct_MethodRef(read_u16(reader), read_u16(reader)),
        PoolTag_InterfaceMethodRef => PoolStruct_InterfaceMethodRef(read_u16(reader), read_u16(reader)),
        PoolTag_String => PoolStruct_String(read_u16(reader)),
        PoolTag_Integer => PoolStruct_Integer(read_u32(reader)),
        PoolTag_Float => PoolStruct_Float(read_u32(reader)),
        PoolTag_Long => PoolStruct_Long(read_u32(reader), read_u32(reader)),
        PoolTag_NameAndType => PoolStruct_NameAndType(read_u16(reader), read_u16(reader)),
        PoolTag_Utf8 => {
            let byts = reader.read_bytes(read_u16(reader) as uint);
            assert str::is_utf8(byts);
            PoolStruct_Utf8(str::from_bytes(byts))
        },
        PoolTag_MethodHandle => PoolStruct_MethodHandle(read_u8(reader), read_u16(reader)),
        PoolTag_MethodType => PoolStruct_MethodType(read_u16(reader)),
        PoolTag_InvokeDynamic => PoolStruct_InvokeDynamic(read_u16(reader), read_u16(reader)),            
        _ => fail fmt!("Unrecognized tag %?", tag)
    };
    ConstantPoolInfo { tag: tag, info: inf }
}

fn read_constant_pool(reader: io::ReaderUtil) -> ~[ConstantPoolInfo] {
    let constant_pool_count = read_u16(reader);
    let mut constant_pool: ~[ConstantPoolInfo] = ~[];
    vec::reserve(&mut constant_pool, constant_pool_count as uint - 1);
    debug!("Running for %? consts", constant_pool_count);
    for iter::repeat(constant_pool_count as uint - 1) {
        constant_pool.push(ConstantPoolInfo(reader));
    }
    constant_pool
}

enum ConstantPoolStructure {
    PoolStruct_Class(u16 /*name_index*/),
    PoolStruct_FieldRef(u16 /*class_index*/, u16 /*name_and_type_index*/),
    PoolStruct_MethodRef(u16 /*class_index*/, u16 /*name_and_type_index*/),
    PoolStruct_InterfaceMethodRef(u16 /*class_index*/, u16 /*name_and_type_index*/),
    PoolStruct_String(u16 /*string_index*/),
    PoolStruct_Integer(u32 /*bytes*/),
    PoolStruct_Float(u32 /*bytes*/),
    PoolStruct_Long(u32 /*high_bytes*/, u32 /*low_bytes*/),
    PoolStruct_NameAndType(u16 /*name_index*/, u16 /*descriptor_index*/),
    //PoolStruct_Utf8(u16 /*length*/, ~[u8] /*bytes*/),
    PoolStruct_Utf8(~str /*str*/),
    PoolStruct_MethodHandle(u8 /*reference_kind*/, u16 /*reference_index*/),
    PoolStruct_MethodType(u16 /*descriptor_index*/),
    PoolStruct_InvokeDynamic(u16 /*bootstrap_method_attr_index*/, u16 /*name_and_type_index*/)
}

struct FieldInfo {
    access_flags: u16,
    name_index: u16,
    descriptor_index: u16,
    attributes: ~[AttributeInfo]
}

fn FieldInfo(constant_pool: &~[ConstantPoolInfo], reader: io::ReaderUtil) -> FieldInfo {
    let access_flags = read_u16(reader);
    let name_index = read_u16(reader);
    let descriptor_index = read_u16(reader);
    let attributes = read_attributes(constant_pool, reader);
    FieldInfo {
        access_flags: access_flags,
        name_index: name_index,
        descriptor_index: descriptor_index,
        attributes: attributes
    }
}

fn read_fields(constant_pool: ~[ConstantPoolInfo], reader: io::ReaderUtil) -> ~[FieldInfo] {
    let field_count = read_u16(reader);
    let mut fields: ~[FieldInfo] = ~[];
    vec::reserve(&mut fields, field_count as uint);
    for iter::repeat(field_count as uint) {
        fields.push(FieldInfo(&constant_pool, reader));
    }
    fields
}


enum FieldAccessFlag {
    FieldAccess_Public = 0x0001,
    FieldAccess_Private = 0x0002,
    FieldAccess_Protected = 0x0004,
    FieldAccess_Static = 0x0008,
    FieldAccess_Final = 0x0010,
    FieldAccess_Volatile = 0x0040,
    FieldAccess_Transient = 0x0080,
    FieldAccess_Synthetic = 0x1000,
    FieldAccess_Enum = 0x4000
}

struct MethodInfo {
    access_flags: u16,
    name_index: u16,
    descriptor_index: u16,
    attributes: ~[AttributeInfo]
}

enum MethodAccessFlag {
    MethodAccess_Public = 0x0001,
    MethodAccess_Private = 0x0002,
    MethodAccess_Protected = 0x0004,
    MethodAccess_Static = 0x0008,
    MethodAccess_Final = 0x0010,
    MethodAccess_Synchronized = 0x0020,
    MethodAccess_Bridge = 0x0040,
    MethodAccess_VarArgs = 0x0080,
    MethodAccess_Native = 0x0100,
    MethodAccess_Abstract = 0x0400,
    MethodAccess_Strict = 0x0800,
    MethodAccess_Synthetic = 0x1000
}

struct AttributeInfo {
    attribute_name_index: u16,
    attribute_length: u32,
    info: AttributeInfoStructure
}

fn read_attributes(constant_pool: &~[ConstantPoolInfo], reader: io::ReaderUtil) -> ~[AttributeInfo] {
    let attribute_count = read_u16(reader);
    let mut attributes: ~[AttributeInfo] = ~[];
    vec::reserve(&mut attributes, attribute_count as uint);
    for iter::repeat(attribute_count as uint) {
        attributes.push(AttributeInfo(constant_pool, reader));
    }
    attributes
}

fn AttributeInfo(constant_pool: &~[ConstantPoolInfo], reader: io::ReaderUtil) -> AttributeInfo {
    let attribute_name_index = read_u16(reader);
    let attribute_length = read_u32(reader);
    //lookup the name
    let attribute_name = &constant_pool[attribute_name_index];
    let inf: AttributeInfoStructure = match attribute_name.info {
        PoolStruct_Utf8(strval) => {
            match strval {
                //TODO: byte length verification please
                ~"ConstantValue" => AttrStruct_ConstantValue(read_u16(reader)),
                ~"Code" => AttrStruct_Code(CodeAttributeInfo(constant_pool, reader)),
                ~"StackMapTable" => AttrStruct_StackMapTable(read_stack_map_table(reader)),
                ~"Exceptions" => AttrStruct_Exceptions(read_u16_vec(reader)),
                ~"InnerClasses" => AttrStruct_InnerClasses(read_inner_class_attributes(reader)),
                ~"EnclosingMethod" => AttrStruct_EnclosingMethod(read_u16(reader), read_u16(reader)),
                ~"Synthetic" => AttrStruct_Synthetic,
                ~"Signature" => AttrStruct_Signature(read_u16(reader)),
                ~"SourceFile" => AttrStruct_SourceFile(read_u16(reader)),
                ~"SourceDebugExtension" => AttrStruct_SourceDebugExtension(reader.read_bytes(attribute_length as uint)),
                ~"LineNumberTable" => AttrStruct_LineNumberTable(read_line_number_table(reader)),
                ~"LocalVariableTable" => AttrStruct_LocalVariableTable(read_local_variable_table(reader)),
                ~"LocalVariableTypeTable" => AttrStruct_LocalVariableTypeTable(read_local_variable_type_table(reader)),
                ~"Deprecated" => AttrStruct_Deprecated,
                ~"RuntimeVisibleAnnotations" => AttrStruct_RuntimeVisibleAnnotations(read_annotations(reader)),
                ~"RuntimeInvisibleAnnotations" => AttrStruct_RuntimeInvisibleAnnotations(read_annotations(reader)),
                ~"RuntimeVisibleParameterAnnotations" => AttrStruct_RuntimeVisibleParameterAnnotations(
                    read_parameter_annotations(reader)),
                ~"RuntimeInvisibleParameterAnnotations" => AttrStruct_RuntimeInvisibleParameterAnnotations(
                    read_parameter_annotations(reader)),
                ~"AnnotationDefault" => AttrStruct_AnnotationDefault(AnnotationElementValue(reader)),
                ~"BootstrapMethods" => AttrStruct_BootstrapMethods(read_bootstrap_methods(reader)),
                _ => {
                    debug!("Warning, unrecognized annotation: %?", strval);
                    AttrStruct_Other(reader.read_bytes(attribute_length as uint))
                }
            }
        },
        _ => fail ~"Attribute not a utf8"
    };
    AttributeInfo {
        attribute_name_index: attribute_name_index,
        attribute_length: attribute_length,
        info: inf
    }
}

enum AttributeInfoStructure {
    AttrStruct_ConstantValue(u16 /*constantvalue_index*/),
    AttrStruct_Code(CodeAttributeInfo),
    AttrStruct_StackMapTable(~[StackMapFrame]),
    AttrStruct_Exceptions(~[u16] /*exception_index_table*/),
    AttrStruct_InnerClasses(~[InnerClassAttributeInfo] /*classes*/),
    AttrStruct_EnclosingMethod(u16 /*class_index*/, u16 /*method_index*/),
    AttrStruct_Synthetic,
    AttrStruct_Signature(u16 /*signature_index*/),
    AttrStruct_SourceFile(u16 /*sourcefile_index*/),
    AttrStruct_SourceDebugExtension(~[u8] /*debug_extension*/),
    AttrStruct_LineNumberTable(~[LineNumberTableInfo] /*line_number_table*/),
    AttrStruct_LocalVariableTable(~[LocalVariableTableInfo] /*local_variable_table*/),
    AttrStruct_LocalVariableTypeTable(~[LocalVariableTypeTableInfo] /*local_variable_type_table*/),
    AttrStruct_Deprecated,
    AttrStruct_RuntimeVisibleAnnotations(~[AnnotationInfo] /*annotations*/),
    AttrStruct_RuntimeInvisibleAnnotations(~[AnnotationInfo] /*annotations*/),
    AttrStruct_RuntimeVisibleParameterAnnotations(~[ParameterAnnotationInfo] /*parameter_annotations*/),
    AttrStruct_RuntimeInvisibleParameterAnnotations(~[ParameterAnnotationInfo] /*parameter_annotations*/),
    AttrStruct_AnnotationDefault(AnnotationElementValue /*default_value*/),
    AttrStruct_BootstrapMethods(~[BootstrapMethodInfo] /*bootstrap_methods*/),
    AttrStruct_Other(~[u8] /*bytes*/)
}

struct CodeAttributeInfo {
    max_stack: u16,
    max_locals: u16,
    code: ~[u8],
    exception_table: ~[ExceptionTableInfo],
    attributes: ~[AttributeInfo]    
}

fn CodeAttributeInfo(constant_pool: &~[ConstantPoolInfo], reader: io::ReaderUtil) -> CodeAttributeInfo {
    let max_stack = read_u16(reader);
    let max_locals = read_u16(reader);
    let code = reader.read_bytes(read_u32(reader) as uint);
    let exception_table = read_exception_table(reader);
    let attributes = read_attributes(constant_pool, reader);
    CodeAttributeInfo {
        max_stack: max_stack,
        max_locals: max_locals,
        code: code,
        exception_table: exception_table,
        attributes: attributes
    }
}

struct ExceptionTableInfo {
    start_pc: u16,
    end_pc: u16,
    handler_pc: u16,
    catch_type: u16
}

fn read_exception_table(reader: io::ReaderUtil) -> ~[ExceptionTableInfo] {
    let exception_table_length = read_u16(reader);
    let mut exception_table: ~[ExceptionTableInfo] = ~[];
    vec::reserve(&mut exception_table, exception_table_length as uint);
    for iter::repeat(exception_table_length as uint) {
        exception_table.push(ExceptionTableInfo {
            start_pc: read_u16(reader),
            end_pc: read_u16(reader),
            handler_pc: read_u16(reader),
            catch_type: read_u16(reader)
        });
    }
    exception_table
}

struct StackMapFrame {
    frame_type: u8,
    info: StackMapFrameType
}

fn read_stack_map_table(reader: io::ReaderUtil) -> ~[StackMapFrame] {
    let stack_map_length = read_u16(reader);
    let mut stack_map_table: ~[StackMapFrame] = ~[];
    vec::reserve(&mut stack_map_table, stack_map_length as uint);
    for iter::repeat(stack_map_length as uint) {
        stack_map_table.push(StackMapFrame(reader));
    }
    stack_map_table
}

fn StackMapFrame(reader: io::ReaderUtil) -> StackMapFrame {
    let frame_type = read_u8(reader);
    let inf = match frame_type {
        0..63 => StackFrame_Same,
        64..127 => StackFrame_SameLocalsStackItem(VerificationTypeInfo(reader)),
        247 => StackFrame_SameLocalsStackItemExtended(read_u16(reader), VerificationTypeInfo(reader)),
        248..250 => StackFrame_Chop(read_u16(reader)),
        251 => StackFrame_SameExtended(read_u16(reader)),
        252..254 => StackFrame_Append(read_u16(reader), read_verification_type_infos_with_count(
            frame_type as uint - 251, reader)),
        255 => StackFrame_Full(read_u16(reader), read_verification_type_infos(reader),
            read_verification_type_infos(reader)),
        _ => fail fmt!("Unrecognized stack frame type: %?", frame_type)
    };
    StackMapFrame { frame_type: frame_type, info: inf }
}

enum StackMapFrameType {
    StackFrame_Same,
    StackFrame_SameLocalsStackItem(VerificationTypeInfo /*stack*/),
    StackFrame_SameLocalsStackItemExtended(u16 /*offset_delta*/, VerificationTypeInfo /*stack*/),
    StackFrame_Chop(u16 /*offset_delta*/),
    StackFrame_SameExtended(u16 /*offset_delta*/),
    StackFrame_Append(u16 /*offset_delta*/, ~[VerificationTypeInfo] /*locals*/),
    StackFrame_Full(u16 /*offset_delta*/, ~[VerificationTypeInfo] /*locals*/, ~[VerificationTypeInfo] /*stack*/)
}

struct VerificationTypeInfo {
    tag: u8,
    info: VariableInfo
}

fn read_verification_type_infos(reader: io::ReaderUtil) -> ~[VerificationTypeInfo] {
    read_verification_type_infos_with_count(read_u16(reader) as uint, reader)
}

fn read_verification_type_infos_with_count(count: uint, reader: io::ReaderUtil) -> ~[VerificationTypeInfo] {
    let mut infos: ~[VerificationTypeInfo] = ~[];
    vec::reserve(&mut infos, count as uint);
    for iter::repeat(count as uint) {
        infos.push(VerificationTypeInfo(reader));
    }
    infos
}

fn VerificationTypeInfo(reader: io::ReaderUtil) -> VerificationTypeInfo {
    let tag = read_u8(reader);
    let inf = match tag {
        0 => Var_Top,
        1 => Var_Integer,
        2 => Var_Float,
        4 => Var_Long,
        3 => Var_Double,
        5 => Var_Null,
        6 => Var_UninitializedThis,
        7 => Var_Object(read_u16(reader)),
        8 => Var_Uninitialized(read_u16(reader)),
        _ => fail fmt!("Unrecognized verification type tag: %?", tag)
    };
    VerificationTypeInfo { tag: tag, info: inf }
}

enum VariableInfo {
    Var_Top,
    Var_Integer,
    Var_Float,
    Var_Long,
    Var_Double,
    Var_Null,
    Var_UninitializedThis,
    Var_Object(u16 /*cpool_index*/),
    Var_Uninitialized(u16 /*offset*/)
}

struct InnerClassAttributeInfo {
    inner_class_info_index: u16,
    outer_class_info_index: u16,
    inner_name_index: u16,
    inner_class_access_flags: u16
}

fn read_inner_class_attributes(reader: io::ReaderUtil) -> ~[InnerClassAttributeInfo] {
    let count = read_u16(reader);
    let mut classes: ~[InnerClassAttributeInfo] = ~[];
    vec::reserve(&mut classes, count as uint);
    for iter::repeat(count as uint) {
        classes.push(InnerClassAttributeInfo {
            inner_class_info_index: read_u16(reader),
            outer_class_info_index: read_u16(reader),
            inner_name_index: read_u16(reader),
            inner_class_access_flags: read_u16(reader)
        });
    }
    classes
}

enum InnerClassAccessFlag {
    InnerClassAccess_Public = 0x0001,
    InnerClassAccess_Private = 0x0002,
    InnerClassAccess_Protected = 0x0004,
    InnerClassAccess_Static = 0x0008,
    InnerClassAccess_Final = 0x0010,
    InnerClassAccess_Interface = 0x0200,
    InnerClassAccess_Abstract = 0x0400,
    InnerClassAccess_Synthetic = 0x1000,
    InnerClassAccess_Annotation = 0x2000,
    InnerClassAccess_Enum = 0x4000
}

struct LineNumberTableInfo {
    start_pc: u16,
    line_number: u16
}

fn read_line_number_table(reader: io::ReaderUtil) -> ~[LineNumberTableInfo] {
    let count = read_u16(reader);
    let mut table: ~[LineNumberTableInfo] = ~[];
    vec::reserve(&mut table, count as uint);
    for iter::repeat(count as uint) {
        table.push(LineNumberTableInfo {
            start_pc: read_u16(reader),
            line_number: read_u16(reader)
        });
    }
    table
}

struct LocalVariableTableInfo {
    start_pc: u16,
    length: u16,
    name_index: u16,
    descriptor_index: u16,
    index: u16
}

fn read_local_variable_table(reader: io::ReaderUtil) -> ~[LocalVariableTableInfo] {
    let count = read_u16(reader);
    let mut table: ~[LocalVariableTableInfo] = ~[];
    vec::reserve(&mut table, count as uint);
    for iter::repeat(count as uint) {
        table.push(LocalVariableTableInfo {
            start_pc: read_u16(reader),
            length: read_u16(reader),
            name_index: read_u16(reader),
            descriptor_index: read_u16(reader),
            index: read_u16(reader)
        });
    }
    table
}

struct LocalVariableTypeTableInfo {
    start_pc: u16,
    length: u16,
    name_index: u16,
    signature_index: u16,
    index: u16
}

fn read_local_variable_type_table(reader: io::ReaderUtil) -> ~[LocalVariableTypeTableInfo] {
    let count = read_u16(reader);
    let mut table: ~[LocalVariableTypeTableInfo] = ~[];
    vec::reserve(&mut table, count as uint);
    for iter::repeat(count as uint) {
        table.push(LocalVariableTypeTableInfo {
            start_pc: read_u16(reader),
            length: read_u16(reader),
            name_index: read_u16(reader),
            signature_index: read_u16(reader),
            index: read_u16(reader)
        });
    }
    table
}

struct AnnotationInfo {
    type_index: u16,
    element_value_pairs: ~[AnnotationElementValuePair]
}

fn read_annotations(reader: io::ReaderUtil) -> ~[AnnotationInfo] {
    let count = read_u16(reader);
    let mut annotations: ~[AnnotationInfo] = ~[];
    vec::reserve(&mut annotations, count as uint);
    for iter::repeat(count as uint) {
        annotations.push(AnnotationInfo(reader));
    }
    annotations
}

fn AnnotationInfo(reader: io::ReaderUtil) -> AnnotationInfo {
    AnnotationInfo {
        type_index: read_u16(reader),
        element_value_pairs: read_element_value_pairs(reader)
    }
}

struct AnnotationElementValuePair {
    element_name_index: u16,
    value: AnnotationElementValue
}

fn read_element_value_pairs(reader: io::ReaderUtil) -> ~[AnnotationElementValuePair] {
    let count = read_u16(reader);
    let mut pairs: ~[AnnotationElementValuePair] = ~[];
    vec::reserve(&mut pairs, count as uint);
    for iter::repeat(count as uint) {
        pairs.push(AnnotationElementValuePair(reader));
    }
    pairs
}

fn AnnotationElementValuePair(reader: io::ReaderUtil) -> AnnotationElementValuePair {
    AnnotationElementValuePair {
        element_name_index: read_u16(reader),
        value: AnnotationElementValue(reader)
    }
}

struct AnnotationElementValue {
    tag: u8,
    value: AnnotationElementValueType
}

fn read_element_values(reader: io::ReaderUtil) -> ~[AnnotationElementValue] {
    let count = read_u16(reader);
    let mut values: ~[AnnotationElementValue] = ~[];
    vec::reserve(&mut values, count as uint);
    for iter::repeat(count as uint) {
        values.push(AnnotationElementValue(reader));
    }
    values
}

fn AnnotationElementValue(reader: io::ReaderUtil) -> AnnotationElementValue {
    let tag = read_u8(reader);
    let value = match tag as char {
        'B'|'C'|'D'|'F'|'I'|'J'|'S'|'Z'|'s' => ElementValueType_Const(read_u16(reader)),
        'e' => ElementValueType_EnumConst(read_u16(reader), read_u16(reader)),
        'c' => ElementValueType_ClassInfo(read_u16(reader)),
        '@' => ElementValueType_Annotation(AnnotationInfo(reader)),
        '[' => ElementValueType_Array(read_element_values(reader)),
        _ => fail fmt!("Unrecognized element value tag: %?", tag) 
    };
    AnnotationElementValue { tag: tag, value: value }
}

enum AnnotationElementValueType {
    ElementValueType_Const(u16 /*const_value_index*/),
    ElementValueType_EnumConst(u16 /*type_name_index*/, u16 /*const_name_index*/),
    ElementValueType_ClassInfo(u16 /*class_info_index*/),
    ElementValueType_Annotation(AnnotationInfo /*annotation*/),
    ElementValueType_Array(~[AnnotationElementValue] /*values*/)
}

struct ParameterAnnotationInfo {
    annotations: ~[AnnotationInfo]
}

fn read_parameter_annotations(reader: io::ReaderUtil) -> ~[ParameterAnnotationInfo] {
    let count = read_u16(reader);
    let mut annotations: ~[ParameterAnnotationInfo] = ~[];
    vec::reserve(&mut annotations, count as uint);
    for iter::repeat(count as uint) {
        annotations.push(ParameterAnnotationInfo { 
            annotations: read_annotations(reader)
        });
    }
    annotations
}

struct BootstrapMethodInfo {
    bootstrap_method_ref: u16,
    bootstrap_arguments: ~[u16]
}

fn read_bootstrap_methods(reader: io::ReaderUtil) -> ~[BootstrapMethodInfo] {
    let count = read_u16(reader);
    let mut methods: ~[BootstrapMethodInfo] = ~[];
    vec::reserve(&mut methods, count as uint);
    for iter::repeat(count as uint) {
        methods.push(BootstrapMethodInfo {
            bootstrap_method_ref: read_u16(reader),
            bootstrap_arguments: read_u16_vec(reader)
        });
    }
    methods
}

fn read_u8(reader: io::ReaderUtil) -> u8 { reader.read_be_uint(1) as u8 }
fn read_u16(reader: io::ReaderUtil) -> u16 { reader.read_be_uint(2) as u16 }
fn read_u32(reader: io::ReaderUtil) -> u32 { reader.read_be_uint(4) as u32 }
fn read_u16_vec(reader: io::ReaderUtil) -> ~[u16] {
    let count = read_u16(reader);
    let mut vec: ~[u16] = ~[];
    vec::reserve(&mut vec, count as uint);
    for iter::repeat(count as uint) {
        vec.push(read_u16(reader));
    }
    vec
}

pub fn ClassFile(reader: io::ReaderUtil) {
    //magic
    let magic = read_u32(reader);
    assert magic == 0xCAFEBABE;

    //versions
    let minor_version = read_u16(reader);
    let major_version = read_u16(reader);
    debug!("Major: %?, Minor: %?", major_version, minor_version);

    //constant pool
    let mut constant_pool = read_constant_pool(reader);
    debug!("Consts: %?", constant_pool);

    //access flags
    let access_flags = read_u16(reader);
    
    //this class
    let this_class = read_u16(reader);

    //super class
    let super_class = read_u16(reader);

    //interfaces
    let interfaces = read_u16_vec(reader);
    debug!("Interfaces: %?", interfaces);

    //fields
    let mut fields = read_fields(constant_pool, reader);
}


