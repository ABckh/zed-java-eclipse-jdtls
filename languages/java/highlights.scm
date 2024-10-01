; Methods

(method_declaration
  name: (identifier) @function.method)
(method_invocation
  name: (identifier) @function.method.call)
(super) @function.builtin

; Parameters
(formal_parameter
  name: (identifier) @variable.parameter)

(catch_formal_parameter
  name: (identifier) @variable.parameter)

(spread_parameter
  (variable_declarator
    name: (identifier) @variable.parameter)) ; int... foo

; Annotations

(annotation
  name: (identifier) @attribute)
(marker_annotation
  name: (identifier) @attribute)

"@" @operator

; Types

(type_identifier) @type

(interface_declaration
  name: (identifier) @type)

(class_declaration
  name: (identifier) @type)

(enum_declaration
  name: (identifier) @type)

((field_access
  object: (identifier) @type)
 (#match? @type "^[A-Z]"))

((scoped_identifier
  scope: (identifier) @type)
 (#match? @type "^[A-Z]"))

((method_invocation
  object: (identifier) @type)
 (#match? @type "^[A-Z]"))

((method_reference
  . (identifier) @type)
 (#match? @type "^[A-Z]"))

(constructor_declaration
  name: (identifier) @type)

(compact_constructor_declaration
  name: (identifier) @type)

((type_identifier) @type.builtin
  (#eq? @type.builtin "var"))

[
  (boolean_type)
  (integral_type)
  (floating_point_type)
  (floating_point_type)
  (void_type)
] @type.builtin

; Fields
(field_declaration
  declarator: (variable_declarator
    name: (identifier) @variable.member))

(field_access
  field: (identifier) @variable.member)

; Variables

((identifier) @constant
 (#match? @constant "^_*[A-Z][A-Z\\d_]+$"))

(field_access
  field: (identifier) @property)

(this) @variable.special

; Literals

[
  (hex_integer_literal)
  (decimal_integer_literal)
  (octal_integer_literal)
  (binary_integer_literal)
] @number

[
  (decimal_floating_point_literal)
  (hex_floating_point_literal)
] @number.float

(string_literal) @string

(escape_sequence) @string.escape

(character_literal) @character

[
  (true)
  (false)
  (null_literal)
] @constant.builtin

[
  (line_comment)
  (block_comment)
] @comment

; Keywords

[
  "abstract"
  "assert"
  "catch"
  "extends"
  "final"
  "finally"
  "implements"
  "instanceof"
  "native"
  "non-sealed"
  "open"
  "private"
  "protected"
  "public"
  "sealed"
  "static"
  "strictfp"
  "synchronized"
  "throw"
  "throws"
  "to"
  "transient"
  "transitive"
  "try"
  "volatile"
  "with"
] @keyword

[
  "return"
  "yield"
] @keyword.return

; Lambda parameter
(inferred_parameters
  (identifier) @variable.parameter) ; (x,y) -> ...

(lambda_expression
  parameters: (identifier) @variable.parameter) ; x -> ...

[
  "record"
  "class"
  "enum"
  "interface"
] @keyword.type

(synchronized_statement
  "synchronized" @keyword)

(modifiers
  "synchronized" @keyword.modifier)

(ternary_expression
  [
    "?"
    ":"
  ] @keyword.conditional.ternary)


; [
;   "abstract"
;   "final"
;   "native"
;   "non-sealed"
;   "open"
;   "private"
;   "protected"
;   "public"
;   "sealed"
;   "static"
;   "strictfp"
;   "transitive"
; ] @keyword.modifier


[
  "transient"
  "volatile"
] @keyword.modifier


"new" @keyword.operator

; Conditionals
[
  "if"
  "else"
  "switch"
  "case"
  "when"
  "default"
] @keyword.conditional


; Loops
[
  "for"
  "while"
  "do"
  "continue"
  "break"
] @keyword.repeat

; Includes
[
  "exports"
  "import"
  "module"
  "opens"
  "package"
  "provides"
  "requires"
  "uses"
] @keyword.import

(import_declaration
  (asterisk
    "*" @character.special))

; Operators
[
  "+"
  ":"
  "++"
  "-"
  "--"
  "&"
  "&&"
  "|"
  "||"
  "!"
  "!="
  "=="
  "*"
  "/"
  "%"
  "<"
  "<="
  ">"
  ">="
  "="
  "-="
  "+="
  "*="
  "/="
  "%="
  "->"
  "^"
  "^="
  "&="
  "|="
  "~"
  ">>"
  ">>>"
  "<<"
  "::"
] @operator
