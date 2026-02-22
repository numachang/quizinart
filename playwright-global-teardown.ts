import { unlinkSync } from "fs";
import { join } from "path";

export default function globalTeardown() {
  try {
    unlinkSync(join(__dirname, ".playwright-port"));
  } catch {
    // File may not exist — that's fine
  }
}
