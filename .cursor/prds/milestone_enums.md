# PRD: milestone_enums

## Goal

Add a dedicated `enum` type to Sifr for simple value enumerations. While literal union types partially fill this role, a proper enum gives namespaced constants, type safety, and exhaustive matching.

## Scope

### Simple enums (no associated data)

```python
enum Color:
    RED
    GREEN
    BLUE
```

- Each variant is a distinct value of type `Color`
- Access via `Color.RED`, `Color.GREEN`, `Color.BLUE`
- Enums are `Eq`, `Hash`, `Clone`, `Debug` by default
- Can be used as dict keys, set members, match subjects

### Enums with integer values

```python
enum HttpStatus:
    OK = 200
    NOT_FOUND = 404
```

- `.value` property returns the integer value

### Enum methods

```python
enum Direction:
    NORTH
    SOUTH
    EAST
    WEST

    def is_vertical(self) -> bool:
        match self:
            case Direction.NORTH | Direction.SOUTH:
                return True
            case _:
                return False
```

### Pattern matching integration

```python
match color:
    case Color.RED:
        return "red"
    case Color.GREEN:
        return "green"
    case Color.BLUE:
        return "blue"
```

## Architecture

### Parser

The `enum` keyword needs to be parsed. We'll use the existing `class` parsing infrastructure since `enum` in Python is typically a class with `Enum` base. However, Sifr uses a custom `enum` keyword syntax.

**Approach**: Parse `enum Name:` as a special class definition with `is_enum = true` flag. The parser already handles class bodies.

Actually, looking at the existing AST, Python's `enum` is typically `class Color(Enum):`. We'll support this pattern since the parser already handles it.

### Type System

Add `Type::Enum` variant or use `Type::Class` with an `is_enum` flag.

**Approach**: Use `Type::Class` with special handling - enum classes are identified by having `Enum` as their base class.

### HIR Changes

- Detect `class Color(Enum):` pattern
- Lower enum variant declarations as class-level constants
- Lower `Color.RED` attribute access as enum variant reference

### Codegen

- Emit Rust `enum` with unit variants
- Emit `#[repr(i64)]` for valued enums
- Auto-derive `Debug`, `Clone`, `PartialEq`, `Eq`, `Hash`
- Emit `.value()` method for valued enums
- Pattern matching: `Color.RED` → `Color::Red`

## Test Plan

- `enum_simple.sifr` - basic enum with variants
- `enum_valued.sifr` - enum with integer values and `.value`
- `enum_methods.sifr` - enum with methods
- `enum_match_exhaustive.sifr` - pattern matching on enum
- `enum_as_dict_key.sifr` - enum as dict key

## Definition of Done

- Simple enums work end-to-end
- Enums with integer values work
- Enum methods work
- Pattern matching on enums works
- All existing E2E tests still pass
- Demo: `demos/milestone_enums_demo.sifr`
