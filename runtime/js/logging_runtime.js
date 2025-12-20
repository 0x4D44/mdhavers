// mdhavers logging runtime
const __mdh_log_state = {
  level: 3,
  filter: "",
  rules: [],
  format: "text",
  timestamps: true,
  sinks: ["stderr"],
  callback: null,
  spans: [],
  nextSpanId: 1,
  memory: [],
};

function __mdh_log_parse_level(level) {
  if (typeof level === "number") {
    return level;
  }
  if (typeof level !== "string") {
    return 3;
  }
  switch (level.toLowerCase()) {
    case "wheesht":
      return 0;
    case "roar":
      return 1;
    case "holler":
      return 2;
    case "blether":
      return 3;
    case "mutter":
      return 4;
    case "whisper":
      return 5;
    default:
      return 3;
  }
}

function __mdh_log_level_name(level) {
  switch (level) {
    case 0:
      return "WHEESHT";
    case 1:
      return "ROAR";
    case 2:
      return "HOLLER";
    case 3:
      return "BLETHER";
    case 4:
      return "MUTTER";
    case 5:
      return "WHISPER";
    default:
      return "BLETHER";
  }
}

function __mdh_log_parse_filter(spec) {
  const parts = String(spec).split(",").map((s) => s.trim()).filter(Boolean);
  let level = __mdh_log_state.level;
  const rules = [];
  for (const part of parts) {
    const idx = part.indexOf("=");
    if (idx !== -1) {
      const target = part.slice(0, idx).trim();
      const lvl = __mdh_log_parse_level(part.slice(idx + 1).trim());
      rules.push({ target, level: lvl });
    } else {
      level = __mdh_log_parse_level(part);
    }
  }
  return { level, rules };
}

function __mdh_log_set_filter(spec) {
  const parsed = __mdh_log_parse_filter(spec);
  __mdh_log_state.filter = String(spec);
  __mdh_log_state.level = parsed.level;
  __mdh_log_state.rules = parsed.rules;
}

function __mdh_log_set_level(level) {
  __mdh_log_state.level = __mdh_log_parse_level(level);
}

function __mdh_log_get_level() {
  return __mdh_log_level_name(__mdh_log_state.level).toLowerCase();
}

function __mdh_log_enabled(level, target) {
  const lvl = __mdh_log_parse_level(level);
  const t = target || "";
  let effective = __mdh_log_state.level;
  let bestLen = -1;
  for (const rule of __mdh_log_state.rules) {
    if (t.startsWith(rule.target) && rule.target.length > bestLen) {
      bestLen = rule.target.length;
      effective = rule.level;
    }
  }
  return lvl <= effective;
}

function __mdh_log_format(record) {
  if (__mdh_log_state.format === "json") {
    return JSON.stringify(record);
  }
  const levelName = __mdh_log_level_name(record.level);
  const ts = __mdh_log_state.timestamps ? new Date().toISOString() : "";
  const parts = [];
  parts.push(`[${levelName}]`);
  if (ts) parts.push(ts);
  if (record.target) parts.push(record.target);
  if (record.file) parts.push(`${record.file}:${record.line || 0}`);
  let msg = String(record.message);
  const fieldKeys = record.fields ? Object.keys(record.fields) : [];
  if (fieldKeys.length) {
    const fieldStr = fieldKeys.map((k) => `${k}=${record.fields[k]}`).join(" ");
    msg = `${msg} ${fieldStr}`;
  }
  if (record.span && record.span.length) {
    msg = `${msg} span=${record.span.join(">")}`;
  }
  if (__mdh_log_state.format === "compact") {
    return `[${levelName}] ${msg}`;
  }
  return `${parts.join(" ")} | ${msg}`;
}

function __mdh_log_event(level, message, fields = {}, target = "") {
  const lvl = __mdh_log_parse_level(level);
  let actualFields = {};
  let actualTarget = target;
  if (arguments.length === 3) {
    if (typeof fields === "string") {
      actualTarget = fields;
    } else if (fields && typeof fields === "object" && !Array.isArray(fields)) {
      actualFields = fields;
    } else {
      throw new Error("log_event() expects fields as a dict or target as a string");
    }
  } else if (arguments.length >= 4) {
    if (fields && typeof fields === "object" && !Array.isArray(fields)) {
      actualFields = fields;
    } else if (fields === null || fields === undefined) {
      actualFields = {};
    } else {
      throw new Error("log_event() expects fields as a dict");
    }
    if (actualTarget !== undefined && actualTarget !== null && typeof actualTarget !== "string") {
      throw new Error("log_event() expects target as a string");
    }
  }
  const tgt = typeof actualTarget === "string" ? actualTarget : "";
  if (!__mdh_log_enabled(lvl, tgt)) {
    return null;
  }
  const record = {
    level: lvl,
    message: String(message),
    target: tgt,
    file: "",
    line: 0,
    fields: actualFields,
    span: __mdh_log_state.spans.map((s) => s.name),
  };
  const formatted = __mdh_log_format(record);
  for (const sink of __mdh_log_state.sinks) {
    if (sink === "stderr") {
      console.error(formatted);
    } else if (sink === "stdout") {
      console.log(formatted);
    } else if (sink === "memory") {
      __mdh_log_state.memory.push(formatted);
    }
  }
  if (typeof __mdh_log_state.callback === "function") {
    try {
      __mdh_log_state.callback(record);
    } catch (_) {
      // swallow callback errors
    }
  }
  return null;
}

function __mdh_log_init(config = {}) {
  if (typeof config !== "object" || config === null) {
    throw new Error("log_init() expects a dict");
  }
  if (config.level !== undefined) {
    __mdh_log_state.level = __mdh_log_parse_level(config.level);
  }
  if (config.filter !== undefined) {
    __mdh_log_set_filter(config.filter);
  }
  if (config.format !== undefined) {
    __mdh_log_state.format = String(config.format);
  }
  if (config.timestamps !== undefined) {
    __mdh_log_state.timestamps = !!config.timestamps;
  }
  if (config.sinks !== undefined) {
    const sinks = Array.isArray(config.sinks) ? config.sinks : [];
    __mdh_log_state.sinks = [];
    __mdh_log_state.callback = null;
    for (const spec of sinks) {
      if (!spec || typeof spec !== "object") {
        continue;
      }
      const kind = spec.kind;
      if (kind === "stderr" || kind === "stdout" || kind === "memory") {
        __mdh_log_state.sinks.push(kind);
      } else if (kind === "callback") {
        if (typeof spec.fn === "function") {
          __mdh_log_state.callback = spec.fn;
        }
      }
    }
    if (!__mdh_log_state.sinks.length) {
      __mdh_log_state.sinks.push("stderr");
    }
  }
  return null;
}

function __mdh_log_span(name, level = "blether", fields = {}, target = "") {
  const span = {
    __mdh_span: true,
    id: __mdh_log_state.nextSpanId++,
    name: String(name),
    level: __mdh_log_parse_level(level),
    target: String(target || ""),
    fields: typeof fields === "object" && fields !== null ? fields : {},
  };
  return span;
}

function __mdh_log_span_enter(span) {
  if (!span || !span.__mdh_span) {
    throw new Error("log_span_enter() expects a span handle");
  }
  __mdh_log_state.spans.push(span);
  return null;
}

function __mdh_log_span_exit(span) {
  if (!span || !span.__mdh_span) {
    throw new Error("log_span_exit() expects a span handle");
  }
  const top = __mdh_log_state.spans.pop();
  if (!top || top.id !== span.id) {
    throw new Error("log_span_exit() got a mismatched span");
  }
  return null;
}

function __mdh_log_span_current() {
  if (!__mdh_log_state.spans.length) {
    return null;
  }
  return __mdh_log_state.spans[__mdh_log_state.spans.length - 1];
}

function __mdh_log_span_in(span, fn) {
  __mdh_log_span_enter(span);
  try {
    return fn();
  } finally {
    __mdh_log_span_exit(span);
  }
}
