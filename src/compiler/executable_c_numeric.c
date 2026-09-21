// SPDX-License-Identifier: Apache-2.0
// Numeric/text behavior derived from Bend 2.0.5, Copyright 2026 HigherOrderCO.
// Portability and bounded output changes: TeamDman. See NOTICE.
#include <math.h>
#include <inttypes.h>

INLINE Term tb_impossible(void) { err_fail("entered an impossible match"); }

INLINE int tb_f32_text(char *buf, f32 value) {
  int n = 0;
  int precision = 0;
  if (value != value) return snprintf(buf, 40, "nan");
  for (; precision < 9; ++precision) {
    n = snprintf(buf, 40, "%.*e", precision, (double)value);
    if (strtof(buf, NULL) == value) break;
  }
  char *exponent = strchr(buf, 'e');
  if (exponent == NULL) return n;
  int power = atoi(exponent + 1);
  if (power >= 21 || power <= -7) {
    size_t offset = (size_t)(exponent - buf);
    n = (int)offset + snprintf(exponent, 40 - offset, "e%c%d", power < 0 ? '-' : '+', abs(power));
  } else if (power <= precision) {
    n = snprintf(buf, 40, "%.*f", precision - power, (double)value);
  } else {
    int sign = *buf == '-';
    memmove(buf + sign + 1, buf + sign + 2, (size_t)precision);
    memset(buf + sign + 1 + precision, '0', (size_t)(power - precision));
    n = sign + 1 + power;
  }
  buf[n] = 0;
  return n;
}

INLINE Term tb_f32_show(Env e, Term value) {
  char buffer[40];
  int length = tb_f32_text(buffer, f32_unbox(value));
  return io_str(e, buffer, (u64)length);
}

INLINE Term tb_f32_read(Env e, Term string) {
  u64 length = 0;
  char *text = io_cstr(e, string, &length);
  char *end = NULL;
  f32 value = strtof(text, &end);
  Term result = length > 0 && *end == 0 ? io_box(e, CID_SOME, f32_rewrap(value), 0) : term_pak(CID_NONE, 0);
  tb_host_free(text);
  return result;
}

static u64 tb_show_bytes;
static u64 tb_show_nodes;

INLINE void tb_show_write(const char *text, size_t length) {
  if (length > 8388608 || tb_show_bytes > 8388608 - length) err_fail("pure output byte budget exhausted");
  tb_show_bytes += length;
  io_out(stdout, text, (u64)length);
}

INLINE void tb_show_text(const char *text) { tb_show_write(text, strlen(text)); }

INLINE void tb_show_step(unsigned depth) {
  tb_tick();
  if (depth > 96 || ++tb_show_nodes > 16384) err_fail("pure output budget exhausted");
}

INLINE void tb_show_character(u64 code, char quote) {
  char buffer[16];
  if (code == (u64)(unsigned char)quote || code == '\\') {
    buffer[0] = '\\'; buffer[1] = (char)code; tb_show_write(buffer, 2);
  } else if (code == 0) tb_show_text("\\0");
  else if (code == '\n') tb_show_text("\\n");
  else if (code == '\r') tb_show_text("\\r");
  else if (code == '\t') tb_show_text("\\t");
  else if (code < 32) {
    int length = snprintf(buffer, sizeof(buffer), "\\u%04x", (unsigned)code);
    tb_show_write(buffer, (size_t)length);
  } else { u64 length = io_utf8(buffer, code); tb_show_write(buffer, (size_t)length); }
}

INLINE void tb_show_native(Env e, Term value, unsigned kind) {
  char buffer[64];
  if (kind == 0 || kind == 1) {
    int length = snprintf(buffer, sizeof(buffer), kind == 0 ? "%" PRIu64 "n" : "%" PRIu64, (uint64_t)value);
    tb_show_write(buffer, (size_t)length);
  } else if (kind == 2) {
    int length = tb_f32_text(buffer, f32_unbox(value));
    char *exp = strchr(buffer, 'e');
    if (strchr(buffer, '.') == NULL && (buffer[0] == '-' ? buffer[1] : buffer[0]) >= '0' && (buffer[0] == '-' ? buffer[1] : buffer[0]) <= '9') {
      size_t at = exp ? (size_t)(exp - buffer) : (size_t)length;
      memmove(buffer + at + 2, buffer + at, (size_t)length - at + 1);
      buffer[at] = '.'; buffer[at + 1] = '0'; length += 2;
    }
    tb_show_write(buffer, (size_t)length);
  } else if (kind == 3) {
    Term field[1]; tb_fields(e, value, CID_CHR, 1, true, field);
    tb_show_text("'"); tb_show_character(field[0], '\''); tb_show_text("'");
  } else {
    tb_show_text("\"");
    while (term_aux(value) == CID_SCON) {
      Loc at;
      if (term_tag(value) != TAG_CTR) err_fail("invalid String in pure output");
      at = term_peek(e, value); tb_span(e, at, 2);
      tb_tick(); tb_show_character(e.mem[at], '"'); value = e.mem[at + 1];
    }
    if (value != term_pak(CID_SNIL, 0)) err_fail("invalid String in pure output");
    tb_show_text("\"");
  }
}
