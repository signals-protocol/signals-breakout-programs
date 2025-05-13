# Range Bet Math Core - TypeScript/JavaScript Guide

This document explains how to use the `range-bet-math-core` npm package in TypeScript/JavaScript environments. This package is a WASM implementation of the core mathematical library for the Signals Breakout Contracts protocol.

## Installation

```bash
npm install range-bet-math-core
# or
yarn add range-bet-math-core
```

## Basic Usage

All functions accept `BigInt` type inputs and return `BigInt` type results.

```typescript
import { calculateBinBuyCost, calculateBinSellCost } from "range-bet-math-core";

// Calculate purchase cost for a single bin
const cost = calculateBinBuyCost(100n, 500n, 1000n);
console.log(`Purchase cost: ${cost}`);

// Calculate sale revenue for a single bin
const revenue = calculateBinSellCost(100n, 500n, 1000n);
console.log(`Sale revenue: ${revenue}`);
```

## Using Multiple Bin Calculation Functions

For multiple bin calculations, you must pass bin arrays using `BigUint64Array`.

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

## Calculating Maximum Purchasable Token Quantity

You can calculate the maximum token quantity purchasable with a given budget.

```typescript
import { calculateXForMultiBins } from "range-bet-math-core";

// Set budget and bin array
const budget = 10000n;
const bins = new BigUint64Array([300n, 400n, 500n]);

// Calculate maximum purchasable token quantity
const x = calculateXForMultiBins(budget, bins, 1000n);
console.log(`Maximum purchasable token quantity: ${x}`);
```

## Using in React

Here's a simple example of using the package in a React application.

```typescript
import React, { useState, useEffect } from "react";
import {
  calculateBinBuyCost,
  calculateMultiBinsBuyCost,
} from "range-bet-math-core";

function BettingCalculator() {
  const [amount, setAmount] = useState(100);
  const [singleBinCost, setSingleBinCost] = useState<string | null>(null);
  const [multiBinsCost, setMultiBinsCost] = useState<string | null>(null);

  useEffect(() => {
    try {
      // Single bin calculation
      const cost = calculateBinBuyCost(BigInt(amount), 500n, 1000n);
      setSingleBinCost(cost.toString());

      // Multiple bins calculation
      const bins = new BigUint64Array([300n, 400n, 500n]);
      const multiCost = calculateMultiBinsBuyCost(BigInt(amount), bins, 1000n);
      setMultiBinsCost(multiCost.toString());
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
        <p>Single bin purchase cost: {singleBinCost}</p>
        <p>Multiple bins purchase cost: {multiBinsCost}</p>
      </div>
    </div>
  );
}

export default BettingCalculator;
```

## Using in Next.js

In Next.js, you should only use this package in client-side code.

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

- All function inputs and return values are of `BigInt` type.
- When passing bin arrays, you must use `BigUint64Array`.
- Runtime errors may occur with invalid inputs (e.g., q > t).
- Always implement error handling.

## TypeScript Type Definitions

The TypeScript type definitions included in the package are:

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
