; ADT definitions
(data_type
    name: (_) @name) @definition.class

; Type family definitions
(type_family
    name: (_) @name) @definition.class

; ADT constructor definitions
(data_type
  constructors: (data_constructors
    constructor: (data_constructor
      constructor: [
        (record name: (_) @name) @definition.function
        (prefix name: (_) @name) @definition.function
      ])))

; ADT record fields
(data_type
  constructors: (data_constructors
    constructor: (data_constructor
      constructor: (record
        fields: (fields
          field: (field
            name: (field_name (variable) @name) @definition.field))))))

; Newtype type name definition
(newtype
  name: (name) @name) @definition.class

; Newtype constructor definition
(newtype_constructor
   name: (constructor) @name @definition.function)

; Newtype record field
(newtype
  constructor: (newtype_constructor
    field: (record
      field: (field name: (_) @name) @definition.field)))

; GADT constructor definitions

(gadt_constructors
    (gadt_constructor
      (constructor) @name) @definition.function)

; Type synonym definitions
(type_synomym
    name: (_) @name) @definition.class

; Function definitions
(declarations
  (function
    name: (variable) @name) @definition.function)

(declarations
  (bind
    name: (variable) @name) @definition.function)

(declarations
  (signature
    name: (variable) @name) @definition.function)

; Class definitions
(class
    name: (_) @name) @definition.interface

(class
  declarations: (class_declarations
    declaration: (signature
      name: (variable) @name) @definition.method))


; Module definitions
(header
    module: (module) @name) @definition.module
