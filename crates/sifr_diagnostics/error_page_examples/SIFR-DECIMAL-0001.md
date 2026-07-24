## Erroneous Code

```python
amount = Decimal("1.25")
```

## How To Fix It

Keep exact decimal values out of binary floating point. Use exact literals or string construction, and do not mix Decimal and BigDecimal in one expression.

## Fixed Code

```python
amount = Decimal("1.25")
```
