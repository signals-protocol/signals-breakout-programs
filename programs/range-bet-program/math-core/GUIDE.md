# Range Bet Math Core - TypeScript/JavaScript Guide

This guide explains how to use the `range-bet-math-core` npm package in JavaScript/TypeScript applications.

## Installation

```bash
npm install range-bet-math-core
# or
yarn add range-bet-math-core
```

## API Overview

All functions accept `BigInt` type inputs and return `BigInt` results. The library provides:

| Function                     | Description                                          |
| ---------------------------- | ---------------------------------------------------- |
| `calculateBinBuyCost`        | Calculate cost to buy tokens in a single bin         |
| `calculateBinSellCost`       | Calculate revenue from selling tokens in a bin       |
| `calculateMultiBinsBuyCost`  | Calculate cost to buy tokens across multiple bins    |
| `calculateMultiBinsSellCost` | Calculate revenue from selling across multiple bins  |
| `calculateXForMultiBins`     | Calculate maximum purchasable tokens within a budget |

## Basic Usage

### Single Bin Calculations

```typescript
import { calculateBinBuyCost, calculateBinSellCost } from "range-bet-math-core";

// Calculate purchase cost for a single bin
const cost = calculateBinBuyCost(100n, 500n, 1000n);
console.log(`Purchase cost: ${cost}`);

// Calculate sale revenue for a single bin
const revenue = calculateBinSellCost(100n, 500n, 1000n);
console.log(`Sale revenue: ${revenue}`);
```

### Multiple Bin Calculations

For multiple bin calculations, you must use `BigUint64Array` for the bin array:

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

### Maximum Token Calculation

Calculate the maximum token quantity purchasable within a budget:

```typescript
import { calculateXForMultiBins } from "range-bet-math-core";

// Set budget and bin array
const budget = 10000n;
const bins = new BigUint64Array([300n, 400n, 500n]);

// Calculate maximum purchasable token quantity
const x = calculateXForMultiBins(budget, bins, 1000n);
console.log(`Maximum purchasable token quantity: ${x}`);
```

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

### Next.js Integration

In Next.js, use the "use client" directive for client-side code:

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

## Error Handling

Always implement proper error handling. Common error scenarios include:

- $q$ is greater than $T$ (bin quantity greater than total supply)
- Division by zero in calculations
- Overflow in logarithm calculation
- Invalid parameters (negative values or non-BigInt values)

```typescript
try {
  const cost = calculateBinBuyCost(100n, 500n, 1000n);
  // Process the result
} catch (error) {
  console.error("Failed to calculate cost:", error);
  // Handle the error appropriately
}
```

## TypeScript Type Definitions

The library includes these TypeScript definitions:

```typescript
export function calculateBinBuyCost(x: bigint, q: bigint, t: bigint): bigint;
export function calculateBinSellCost(x: bigint, q: bigint, t: bigint): bigint;
export function calculateMultiBinsBuyCost(
  x: bigint,
  qs: BigUint64Array,
  t: bigint
): bigint;
export function calculateMultiBinsSellCost(
  x: bigint,
  qs: BigUint64Array,
  t: bigint
): bigint;
export function calculateXForMultiBins(
  budget: bigint,
  qs: BigUint64Array,
  t: bigint
): bigint;
```

## Important Notes

- All function inputs and return values are of `BigInt` type
- When passing bin arrays, you must use `BigUint64Array`
- Runtime errors occur with invalid inputs (e.g., q > t)
- The package is designed for browser environments
- For server-side rendering, ensure proper WebAssembly support

## Mathematical Model

The core formula implemented by this library is:

$$\int_{t=0}^{x} \frac{q+t}{T+t} \, dt = x + (q-T) \ln\left(\frac{T+x}{T}\right)$$

Where:

- $q$: Current token quantity in the bin
- $T$: Total token supply in the market
- $x$: Token quantity to purchase

This formula has several important properties:

1. The cost increases as the bin quantity increases, making popular bets more expensive
2. The cost decreases relative to total market supply, balancing the impact of large markets
3. The cost is always positive for valid inputs
4. When $q=0$ (new bin), the cost is less than $x$, providing an incentive for early participants
5. The formula produces a probability distribution over all possible price ranges

> **Note**: The mathematical implementation uses floating-point calculations internally while maintaining an integer-based API to ensure accuracy in logarithmic operations.

## Related Documentation

- [Mathematical Model](../../../docs/math.md) - Detailed explanation of the mathematical formulas
- [WASM Package README](../pkg-wasm/README.md) - npm package documentation
- [API Reference](../../../docs/api-reference.md) - Complete API reference
