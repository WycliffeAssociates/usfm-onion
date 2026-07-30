import {
  FINDING_FIELD,
  FINDING_SECTION_RULES_VERSION,
  LINT_CODES,
  PARAM_CONTRACTS,
  TOKEN_KIND,
} from "usfm-onion-web/wire-schema";

const checks = [
  ["LINT_CODES.length", LINT_CODES.length, 33],
  ["TOKEN_KIND.Text", TOKEN_KIND.Text, 8],
  ["FINDING_SECTION_RULES_VERSION", FINDING_SECTION_RULES_VERSION, 1],
  ["FINDING_FIELD.length", FINDING_FIELD.length, 9],
  ["PARAM_CONTRACTS.length", PARAM_CONTRACTS.length, 25],
];

const failures = checks.filter(([, actual, expected]) => actual !== expected);
if (failures.length > 0) {
  for (const [name, actual, expected] of failures) {
    console.error(`${name}: expected ${expected}, got ${actual}`);
  }
  process.exit(1);
}

console.log("wire-schema import shape conformance passed");
