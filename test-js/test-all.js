import * as childProcess from 'child_process';
import * as fsSync from 'fs';
import * as fs from 'fs/promises';
import * as path from 'path';
import * as url from 'url';

const __dirname = path.dirname(url.fileURLToPath(import.meta.url));

const srcDir = path.join(__dirname, 'src');

const src = await fs.readdir(srcDir);

const tests = src
  .map((test) => [path.join(srcDir, test), test])
  .filter(([testPath]) => {
    const stats = fsSync.statSync(testPath);
    return !stats.isDirectory();
  });

let done = 0;
const successes = [];
const failures = [];

function maybeDone() {
  if (done === tests.length) {
    for (const success of successes) {
      console.log(`✔️ Test ${success} successful`);
    }
    for (const { name, stderr } of failures) {
      console.log(
        `------------------- stderr test ${name} -------------------`
      );
      console.log(stderr);
      console.log(`------------------- stderr test ${name} -------------------
❌ Test ${name} failed`);
    }

    if (failures.length > 0) {
      process.exit(1);
    }
  }
}

function runTest(path, name) {
  childProcess.exec(`node ${path}`, {}, (error, _, stderr) => {
    if (!error) {
      successes.push(name);
    } else {
      failures.push({ name, stderr });
    }
    done += 1;
    maybeDone();
  });
}

for (const [test, name] of tests) {
  runTest(test, name);
}
