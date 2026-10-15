import { clsx, type ClassValue } from "clsx";
import { twMerge } from "tailwind-merge";

/** Class-name builder: conditionals + Tailwind conflict resolution. */
export const cn = (...inputs: ClassValue[]): string => twMerge(clsx(inputs));
