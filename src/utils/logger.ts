interface Logger {
  debug(message: string, ...args: unknown[]): void;
  info(message: string, ...args: unknown[]): void;
  warn(message: string, ...args: unknown[]): void;
  error(message: string, ...args: unknown[]): void;
  fatal(message: string, ...args: unknown[]): void;
}

class SimpleLogger implements Logger {
  private getTimestamp(): string {
    return new Date().toISOString();
  }

  private formatMessage(level: string, message: string, args: unknown[]): string {
    const timestamp = this.getTimestamp();
    const argsStr = args.length > 0 ? ' ' + args.map(arg => 
      typeof arg === 'object' ? JSON.stringify(arg) : String(arg)
    ).join(' ') : '';
    return `[${timestamp}] ${level.padEnd(5)} ${message}${argsStr}`;
  }

  debug(message: string, ...args: unknown[]): void {
    if (process.env.NODE_ENV === 'development' || process.env.NOX_LOG_LEVEL === 'debug') {
      console.debug(this.formatMessage('DEBUG', message, args));
    }
  }

  info(message: string, ...args: unknown[]): void {
    console.log(this.formatMessage('INFO', message, args));
  }

  warn(message: string, ...args: unknown[]): void {
    console.warn(this.formatMessage('WARN', message, args));
  }

  error(message: string, ...args: unknown[]): void {
    console.error(this.formatMessage('ERROR', message, args));
  }

  fatal(message: string, ...args: unknown[]): void {
    console.error(this.formatMessage('FATAL', message, args));
  }
}

export const logger = new SimpleLogger();