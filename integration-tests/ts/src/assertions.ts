import { NEAR, NearAccount } from "near-workspaces-ava";

export async function expect(action: () => {}, ...assertions: Assertion[]) {
  const before = await Promise.all(assertions.map((a) => a.before()));

  await action();

  const after = await Promise.all(assertions.map((a) => a.after()));

  const expected = assertions.map((a) => a.expected());
  const actual = assertions.map((a, i) => a.actual(before[i], after[i]));
  return { before, after, expected, actual };
}

type Assertion = {
  //name: string;
  before: () => any;
  after: () => any;
  expected: () => any;
  actual: (before: any, after: any) => any;
};

export function toChangeNearBalance(
  account: NearAccount,
  amount: string,
  precision: string = "1 N"
): Assertion {
  const value = NEAR.parse(amount);
  const p = NEAR.parse(precision);
  const description = `${account.accountId} NEAR balance changed by `;

  return {
    //name: `${this.name}(${account.accountId},${value.toString()})`,
    before: async () => await account.availableBalance(),
    after: async () => await account.availableBalance(),
    expected: () => `${description}${value.divRound(p).mul(p).toString()}`,
    actual: (before: NEAR, after: NEAR) =>
      `${description}${after.sub(before).divRound(p).mul(p).toString()}`,
  };
}
