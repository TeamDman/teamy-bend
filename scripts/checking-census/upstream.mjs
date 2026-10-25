// SPDX-License-Identifier: MPL-2.0
import { writeFileSync } from 'node:fs';
import { pathToFileURL } from 'node:url';
const Bend = await import(pathToFileURL(process.argv[2]).href);
let phase = 'load';
let outcome = 'accepted';
let incomplete = false;
try {
  const book = Bend.book_nil();
  await Bend.book_load(book, process.argv[3], '', new Map());
  phase = 'check';
  Bend.book_valid(book);
  const holes = book.hols + book.open;
  if (holes > 0) { incomplete = true; throw `Error: ${holes} TODO${holes === 1 ? '' : 's'} found.\nThe code is incomplete, and not a valid proof yet.`; }
  phase = 'accepted';
} catch (error) {
  outcome = error?.$ === 'Err' ? 'language_rejection' : incomplete ? 'incomplete_proof' : error instanceof RangeError ? 'host_stack_failure' : 'unexpected_host_failure';
  const text = error instanceof RangeError
    ? 'Error: the machine stack overflowed (a deep recursion, or a literal too large to expand)'
    : error?.$ === 'Err' ? Bend.err_show(error) : String(error);
  process.stderr.write(text + '\n');
  process.exitCode = 1;
} finally {
  writeFileSync(process.argv[4], JSON.stringify({phase, outcome})+'\n');
}
