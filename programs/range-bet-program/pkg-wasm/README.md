# range-bet-math-core

[![npm version](https://img.shields.io/npm/v/range-bet-math-core.svg)](https://www.npmjs.com/package/range-bet-math-core)
[![License: ISC](https://img.shields.io/badge/License-ISC-blue.svg)](https://opensource.org/licenses/ISC)

A WebAssembly-powered library that implements the mathematical functions for the Signals Protocol. This package provides cost calculation functions for prediction market betting.

## Installation

```bash
npm install range-bet-math-core
# or
yarn add range-bet-math-core
```

## API Reference

| Function                     | Description                               | Parameters                                      | Return Type |
| ---------------------------- | ----------------------------------------- | ----------------------------------------------- | ----------- |
| `calculateBinBuyCost`        | Calculate cost to buy tokens in a bin     | `x: bigint, q: bigint, t: bigint`               | `bigint`    |
| `calculateBinSellCost`       | Calculate revenue from selling tokens     | `x: bigint, q: bigint, t: bigint`               | `bigint`    |
| `calculateMultiBinsBuyCost`  | Calculate cost for multiple bins          | `x: bigint, qs: BigUint64Array, t: bigint`      | `bigint`    |
| `calculateMultiBinsSellCost` | Calculate revenue for multiple bins       | `x: bigint, qs: BigUint64Array, t: bigint`      | `bigint`    |
| `calculateXForMultiBins`     | Find max tokens purchasable within budget | `budget: bigint, qs: BigUint64Array, t: bigint` | `bigint`    |

Where:

- `x`: Amount of tokens to purchase/sell
- `q`: Current token quantity in the bin
- `qs`: Array of token quantities in multiple bins
- `t`: Total token quantity in the market
- `budget`: Available budget for purchasing tokens

## Usage Examples

### Basic Cost Calculation

```typescript
import { calculateBinBuyCost } from "range-bet-math-core";

// Calculate purchase cost
const cost = calculateBinBuyCost(100n, 500n, 1000n);
console.log(`Purchase cost: ${cost}`);
```

### Multiple Bins Calculation

```typescript
import { calculateMultiBinsBuyCost } from "range-bet-math-core";

// Create bin array using BigUint64Array (required)
const bins = new BigUint64Array([300n, 400n, 500n]);

// Calculate purchase cost across multiple bins
const cost = calculateMultiBinsBuyCost(100n, bins, 1000n);
console.log(`Multiple bins purchase cost: ${cost}`);
```

### Finding Maximum Purchasable Amount

```typescript
import { calculateXForMultiBins } from "range-bet-math-core";

const budget = 10000n;
const bins = new BigUint64Array([300n, 400n, 500n]);

// Calculate maximum tokens purchasable with budget
const x = calculateXForMultiBins(budget, bins, 1000n);
console.log(`Maximum purchasable: ${x} tokens`);
```

## React Integration Example

```typescript
import React, { useState, useEffect } from "react";
import { calculateBinBuyCost } from "range-bet-math-core";

function BettingCalculator() {
  const [amount, setAmount] = useState(100);
  const [cost, setCost] = useState<string | null>(null);

  useEffect(() => {
    try {
      const calculatedCost = calculateBinBuyCost(BigInt(amount), 500n, 1000n);
      setCost(calculatedCost.toString());
    } catch (error) {
      console.error("Calculation error:", error);
    }
  }, [amount]);

  return (
    <div>
      <h2>Betting Cost Calculator</h2>
      <input
        type="number"
        value={amount}
        onChange={(e) => setAmount(Number(e.target.value))}
        min="1"
      />
      <div>
        <p>Purchase cost: {cost}</p>
      </div>
    </div>
  );
}
```

## Next.js Integration Example

```typescript
"use client"; // Important: only use in client components

import { useState, useEffect } from "react";
import { calculateBinBuyCost } from "range-bet-math-core";

export default function BettingComponent() {
  const [cost, setCost] = useState<string | null>(null);

  useEffect(() => {
    try {
      const calculatedCost = calculateBinBuyCost(100n, 500n, 1000n);
      setCost(calculatedCost.toString());
    } catch (error) {
      console.error("Calculation error:", error);
    }
  }, []);

  return (
    <div>
      <h1>Betting Cost</h1>
      {cost ? <p>Purchase cost: {cost}</p> : <p>Calculating...</p>}
    </div>
  );
}
```

## Important Notes

- All functions require `BigInt` inputs and return `BigInt` values
- Arrays must be passed as `BigUint64Array` instances
- This package is for client-side use (browsers); server-side requires additional configuration
- Always implement error handling as invalid inputs will throw errors
- Common error cases: `q > t`, division by zero, overflow in calculations

## Mathematical Background

This library implements the $(q+t)/(T+t)$ integral formula:

$$\int_{t=0}^{x} \frac{q+t}{T+t} \, dt = x + (q-T) \ln\left(\frac{T+x}{T}\right)$$

Where:

- $q$: Current token quantity in the bin
- $T$: Total token supply in the market
- $x$: Token quantity to purchase

> **Note**: While the API uses integer values (BigInt), the underlying WASM module performs floating-point calculations internally for accurate logarithmic operations.

For a detailed explanation of the mathematical model:

- [Mathematical Model Documentation](../../docs/math.md)
- [Implementation Details](../math-core/README.md)

## Additional Documentation

- [TypeScript Guide](../math-core/GUIDE.md) - More detailed usage guide
- [Signals Protocol](https://github.com/signals-protocol/signals) - Main project

## License

ISC
