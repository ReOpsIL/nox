/**
 * Priority Queue implementation
 * Used for message queuing with priority-based processing
 */
export class PriorityQueue<T> {
  private items: T[] = [];
  private comparator: (a: T, b: T) => number;

  /**
   * Create a new priority queue
   * @param comparator Function to compare two items. Should return a negative number if a has higher priority than b,
   *                   zero if they have the same priority, and a positive number if b has higher priority than a.
   */
  constructor(comparator: (a: T, b: T) => number) {
    this.comparator = comparator;
  }

  /**
   * Add an item to the queue
   */
  enqueue(item: T): void {
    // Add the item to the end
    this.items.push(item);
    
    // Sort the queue based on priority
    this.items.sort(this.comparator);
  }

  /**
   * Remove and return the highest priority item
   */
  dequeue(): T | undefined {
    return this.items.shift();
  }

  /**
   * Peek at the highest priority item without removing it
   */
  peek(): T | undefined {
    return this.items[0];
  }

  /**
   * Check if the queue is empty
   */
  isEmpty(): boolean {
    return this.items.length === 0;
  }

  /**
   * Get the number of items in the queue
   */
  size(): number {
    return this.items.length;
  }

  /**
   * Clear the queue
   */
  clear(): void {
    this.items = [];
  }

  /**
   * Convert the queue to an array
   * Note: This returns a copy of the internal array to prevent direct modification
   */
  toArray(): T[] {
    return [...this.items];
  }

  /**
   * Find an item in the queue
   * @param predicate Function to test each item
   * @returns The first item that satisfies the predicate, or undefined if no items satisfy it
   */
  find(predicate: (item: T) => boolean): T | undefined {
    return this.items.find(predicate);
  }

  /**
   * Remove an item from the queue
   * @param predicate Function to test each item
   * @returns True if an item was removed, false otherwise
   */
  remove(predicate: (item: T) => boolean): boolean {
    const index = this.items.findIndex(predicate);
    if (index !== -1) {
      this.items.splice(index, 1);
      return true;
    }
    return false;
  }
}