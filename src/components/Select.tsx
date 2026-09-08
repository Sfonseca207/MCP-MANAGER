import clsx from "clsx";
import { ChevronDown } from "lucide-react";

export interface SelectOption<T extends string = string> {
  value: T;
  label: string;
}

interface SelectProps<T extends string = string> {
  value: T;
  onChange: (value: T) => void;
  options: SelectOption<T>[];
  className?: string;
  id?: string;
}

export function Select<T extends string = string>({
  value,
  onChange,
  options,
  className,
  id,
}: SelectProps<T>) {
  return (
    <div className={clsx("relative", className)}>
      <select
        id={id}
        value={value}
        onChange={(e) => onChange(e.target.value as T)}
        className="w-full cursor-pointer appearance-none rounded-lg border border-[var(--border)] bg-[var(--bg)] py-2 pl-3 pr-9 text-sm outline-none transition-colors hover:border-[var(--text-muted)] focus:border-[var(--accent)] focus:ring-1 focus:ring-[var(--accent)]"
      >
        {options.map((option) => (
          <option key={option.value} value={option.value}>
            {option.label}
          </option>
        ))}
      </select>
      <ChevronDown
        aria-hidden
        className="pointer-events-none absolute right-2.5 top-1/2 h-4 w-4 -translate-y-1/2 text-[var(--text-muted)]"
      />
    </div>
  );
}

interface SegmentedControlProps<T extends string = string> {
  value: T;
  onChange: (value: T) => void;
  options: SelectOption<T>[];
  className?: string;
  size?: "sm" | "md";
}

export function SegmentedControl<T extends string = string>({
  value,
  onChange,
  options,
  className,
  size = "md",
}: SegmentedControlProps<T>) {
  return (
    <div
      className={clsx(
        "flex rounded-lg border border-[var(--border)] bg-[var(--bg-elevated)] p-0.5",
        className,
      )}
    >
      {options.map((option) => (
        <button
          key={option.value}
          type="button"
          onClick={() => onChange(option.value)}
          className={clsx(
            "flex-1 rounded-md transition-colors",
            size === "sm" ? "px-2.5 py-1 text-xs" : "px-3 py-1.5 text-sm",
            value === option.value
              ? "bg-[var(--accent)] text-[var(--on-accent)]"
              : "text-[var(--text-muted)] hover:text-[var(--text)]",
          )}
        >
          {option.label}
        </button>
      ))}
    </div>
  );
}
