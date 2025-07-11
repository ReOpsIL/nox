import * as fs from 'fs/promises';
import * as path from 'path';

export class FileUtils {
  static async ensureDirectory(dirPath: string): Promise<void> {
    try {
      await fs.access(dirPath);
    } catch {
      await fs.mkdir(dirPath, { recursive: true });
    }
  }

  static async exists(filePath: string): Promise<boolean> {
    try {
      await fs.access(filePath);
      return true;
    } catch {
      return false;
    }
  }

  static async readJson<T>(filePath: string): Promise<T> {
    const content = await fs.readFile(filePath, 'utf-8');
    return JSON.parse(content) as T;
  }

  static async writeJson(filePath: string, data: unknown, pretty = true): Promise<void> {
    const content = pretty ? JSON.stringify(data, null, 2) : JSON.stringify(data);
    await this.ensureDirectory(path.dirname(filePath));
    await fs.writeFile(filePath, content, 'utf-8');
  }

  static async copyFile(source: string, destination: string): Promise<void> {
    await this.ensureDirectory(path.dirname(destination));
    await fs.copyFile(source, destination);
  }

  static async deleteFile(filePath: string): Promise<void> {
    try {
      await fs.unlink(filePath);
    } catch (error) {
      // Ignore if file doesn't exist
      if ((error as NodeJS.ErrnoException).code !== 'ENOENT') {
        throw error;
      }
    }
  }

  /**
   * Recursively delete a directory and all its contents
   */
  static async deleteDirectory(dirPath: string): Promise<void> {
    try {
      const exists = await this.exists(dirPath);
      if (!exists) {
        return; // Directory doesn't exist, nothing to do
      }

      const stats = await fs.stat(dirPath);
      if (!stats.isDirectory()) {
        await this.deleteFile(dirPath);
        return;
      }

      // Read directory contents
      const files = await fs.readdir(dirPath);

      // Delete all files and subdirectories
      for (const file of files) {
        const filePath = path.join(dirPath, file);
        const fileStats = await fs.stat(filePath);

        if (fileStats.isDirectory()) {
          // Recursively delete subdirectory
          await this.deleteDirectory(filePath);
        } else {
          // Delete file
          await fs.unlink(filePath);
        }
      }

      // Delete the empty directory
      await fs.rmdir(dirPath);
    } catch (error) {
      // Ignore if directory doesn't exist
      if ((error as NodeJS.ErrnoException).code !== 'ENOENT') {
        throw error;
      }
    }
  }

  static async listFiles(dirPath: string, extension?: string): Promise<string[]> {
    try {
      const files = await fs.readdir(dirPath);
      if (extension) {
        return files.filter(file => file.endsWith(extension));
      }
      return files;
    } catch {
      return [];
    }
  }

  static async getFileStats(filePath: string): Promise<Awaited<ReturnType<typeof fs.stat>> | null> {
    try {
      return await fs.stat(filePath);
    } catch {
      return null;
    }
  }

  static async readFileLines(filePath: string): Promise<string[]> {
    const content = await fs.readFile(filePath, 'utf-8');
    return content.split('\n');
  }

  static async appendToFile(filePath: string, content: string): Promise<void> {
    await this.ensureDirectory(path.dirname(filePath));
    await fs.appendFile(filePath, content);
  }

  static sanitizeFileName(filename: string): string {
    return filename
      .replace(/[^a-zA-Z0-9-_\.]/g, '_')
      .replace(/_{2,}/g, '_')
      .replace(/^_+|_+$/g, '');
  }

  static getRelativePath(from: string, to: string): string {
    return path.relative(from, to);
  }

  static joinPaths(...paths: string[]): string {
    return path.join(...paths);
  }
}
