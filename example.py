"""
Example: How to use BufferVault programmatically
This demonstrates the core functionality without GUI
"""
from clipboard_monitor import ClipboardMonitor
from storage import StorageManager
from config import Config
import time

def example_basic_usage():
    """Basic usage example"""
    print("=" * 60)
    print("BufferVault - Basic Usage Example")
    print("=" * 60)
    
    # Create storage manager with encryption
    storage = StorageManager(
        storage_path='example_vault',
        encryption_enabled=True
    )
    
    # Clear any existing history
    storage.clear_history()
    
    # Simulate clipboard operations by adding entries
    print("\n1. Adding clipboard entries...")
    storage.add_entry("Hello, World!")
    storage.add_entry("import this")
    storage.add_entry("Python is awesome")
    storage.add_entry("https://github.com/odaiko42/bufferVault")
    
    # Get and display history
    print("\n2. Current clipboard history:")
    history = storage.get_history()
    for i, entry in enumerate(history):
        print(f"   {i}. [{entry.get_display_time()}] {entry.get_preview()}")
    
    # Search functionality
    print("\n3. Searching for 'Python':")
    results = storage.search_history("Python")
    for idx, entry in results:
        print(f"   Found at index {idx}: {entry.get_preview()}")
    
    # Get specific entry
    print("\n4. Getting specific entry (index 1):")
    entry = storage.get_entry(1)
    if entry:
        print(f"   Content: {entry.content}")
    
    # Statistics
    print("\n5. Storage statistics:")
    stats = storage.get_stats()
    print(f"   Total entries: {stats['total_entries']}")
    print(f"   Encryption enabled: {stats['encryption_enabled']}")
    print(f"   Storage path: {stats['storage_path']}")
    
    print("\n" + "=" * 60)
    print("✓ Example completed successfully!")
    print("=" * 60)


def example_with_monitor():
    """Example using clipboard monitor (doesn't actually monitor in this demo)"""
    print("\n" + "=" * 60)
    print("BufferVault - Monitor Example")
    print("=" * 60)
    
    # Create monitor
    monitor = ClipboardMonitor()
    
    # Manually add some entries to demonstrate
    print("\n1. Simulating clipboard copies...")
    monitor.storage.add_entry("Copy 1: Hello")
    time.sleep(0.1)
    monitor.storage.add_entry("Copy 2: World")
    time.sleep(0.1)
    monitor.storage.add_entry("Copy 3: BufferVault")
    
    # Get history
    print("\n2. Recent clipboard history (3 items):")
    history = monitor.get_history(limit=3)
    for i, entry in enumerate(history):
        print(f"   {i}. {entry.get_preview(50)}")
    
    # Simulate restoring to clipboard
    print("\n3. Simulating paste operation (index 2):")
    entry = monitor.storage.get_entry(2)
    if entry:
        print(f"   Would paste: '{entry.content}'")
    
    print("\n" + "=" * 60)


if __name__ == '__main__':
    example_basic_usage()
    example_with_monitor()
