# range-bet-math-core

[![npm version](https://img.shields.io/npm/v/range-bet-math-core.svg)](https://www.npmjs.com/package/range-bet-math-core)
[![License: ISC](https://img.shields.io/badge/License-ISC-blue.svg)](https://opensource.org/licenses/ISC)

A WebAssembly-powered library providing the mathematical core for the Signals Breakout Contracts protocol. This library implements essential mathematical functions for calculating betting costs and revenues using the (q+t)/(T+t) integral formula.

## Installation

```bash
npm install range-bet-math-core
# or
yarn add range-bet-math-core
```

## Usage

The library provides a set of functions for calculating purchase costs and sale revenues in prediction markets. All functions accept and return `BigInt` values.

### Basic Functions

```typescript
import { calculateBinBuyCost, calculateBinSellCost } from "range-bet-math-core";

// Calculate purchase cost for a single bin
const cost = calculateBinBuyCost(100n, 500n, 1000n);
console.log(`Purchase cost: ${cost}`);

// Calculate sale revenue for a single bin
const revenue = calculateBinSellCost(100n, 500n, 1000n);
console.log(`Sale revenue: ${revenue}`);
```

### Multiple Bin Operations

For multiple bin calculations, you must pass bin arrays using `BigUint64Array`:

```typescript
import {
  calculateMultiBinsBuyCost,
  calculateMultiBinsSellCost,
} from "range-bet-math-core";

// Create bin array using BigUint64Array
const bins = new BigUint64Array([300n, 400n, 500n]);

// Calculate purchase cost across multiple bins
const cost = calculateMultiBinsBuyCost(100n, bins, 1000n);
console.log(`Multiple bins purchase cost: ${cost}`);

// Calculate sale revenue across multiple bins
const revenue = calculateMultiBinsSellCost(50n, bins, 1000n);
console.log(`Multiple bins sale revenue: ${revenue}`);
```

### Calculate Maximum Purchasable Quantity

You can calculate the maximum token quantity purchasable with a given budget:

```typescript
import { calculateXForMultiBins } from "range-bet-math-core";

// Set budget and bin array
const budget = 10000n;
const bins = new BigUint64Array([300n, 400n, 500n]);

// Calculate maximum purchasable token quantity
const x = calculateXForMultiBins(budget, bins, 1000n);
console.log(`Maximum purchasable token quantity: ${x}`);
```

## API Reference

### Core Functions

| Function                     | Description                                                 | Parameters                                      | Return Type |
| ---------------------------- | ----------------------------------------------------------- | ----------------------------------------------- | ----------- |
| `calculateBinBuyCost`        | Calculates cost to buy tokens in a single bin               | `x: bigint, q: bigint, t: bigint`               | `bigint`    |
| `calculateBinSellCost`       | Calculates revenue from selling tokens in a single bin      | `x: bigint, q: bigint, t: bigint`               | `bigint`    |
| `calculateMultiBinsBuyCost`  | Calculates total cost to buy tokens across multiple bins    | `x: bigint, qs: BigUint64Array, t: bigint`      | `bigint`    |
| `calculateMultiBinsSellCost` | Calculates revenue from selling tokens across multiple bins | `x: bigint, qs: BigUint64Array, t: bigint`      | `bigint`    |
| `calculateXForMultiBins`     | Finds maximum purchasable token quantity within a budget    | `budget: bigint, qs: BigUint64Array, t: bigint` | `bigint`    |

Parameters:

- `x`: Amount of tokens to purchase/sell
- `q`: Current token quantity in the bin
- `qs`: Array of token quantities in multiple bins
- `t`: Total token quantity in the market
- `budget`: Available budget for purchasing tokens

## Framework Integration

### React Example

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

### Next.js Example

```typescript
"use client";

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

- All function inputs and return values are of `BigInt` type
- When passing bin arrays, you must use `BigUint64Array`
- Runtime errors may occur with invalid inputs (e.g., q > t)
- Always implement error handling

## Mathematical Model

The core price formula is an integral:

```
Cost = ∫(q+t)/(T+t) dt, from t=0 to t=x
```

Which evaluates to:

```
Cost = x + (q-T)*ln((T+x)/T)
```

Where:

- `q`: Current token quantity in the bin
- `T`: Total token supply in the market
- `x`: Token quantity to purchase

## License

ISC
