# milestone_decorators — Decorators and Variadics

## 1. Product Requirements

### Objective
Add user-defined function decorators and variadic arguments to Sifr. Decorators enable function wrapping patterns, and variadics enable flexible function signatures.

### Scope

**In Scope:**
- Simple function decorators: `@decorator` applied to functions
- Decorator wrapping: decorator function receives the decorated function and returns a wrapper
- Stacked decorators: multiple `@decorator` applied in order
- `*args: tuple` for variadic positional arguments
- `**kwargs: dict` for variadic keyword arguments

**Out of Scope (deferred):**
- Decorator factories (decorators with arguments like `@decorator(arg)`)
- Class decorators
- `@property` getter/setter (partially in milestone_inheritance)
- Parameterized decorators

### Acceptance Criteria
- AC-1: `@decorator` wraps a function, calling decorator(func) at definition time
- AC-2: Stacked decorators apply in bottom-up order
- AC-3: `*args` collects extra positional arguments into a tuple
- AC-4: `**kwargs` collects extra keyword arguments into a dict

## 2. Solution Design

### 2.1 Decorators
Since Sifr compiles to Rust (not interpreted), decorators are applied at compile time:
- A decorator `@dec` on `def func()` generates: the original function with a modified name, and a new function `func` that calls `dec(original_func)`
- For simple logging/timing decorators, the pattern is: decorator returns a closure that wraps the original

### 2.2 Practical Approach
Given the complexity of full decorator support (requires higher-order functions returning closures), we'll implement a simpler but useful subset:
- `@classmethod` and `@staticmethod` are already handled (milestone_inheritance)
- Add support for recognizing custom decorators and emitting them as function attributes/wrappers
- For this milestone, focus on the decorator syntax recognition and basic wrapping

### 2.3 Testing Strategy
**E2E pass tests:**
- `decorator_basic.sifr` — simple decorator wrapping
- `args_basic.sifr` — *args variadic arguments

**Demo:** `milestone_decorators_demo.sifr`
