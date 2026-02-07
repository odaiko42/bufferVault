"""
BufferVault - Encrypted Clipboard History Manager for Windows 10/11

A lightweight and secure system tool that automatically intercepts every copy 
operation and keeps an encrypted history of all copied items.
"""
import sys
import argparse
from gui import main as gui_main
from clipboard_monitor import ClipboardMonitor
from config import Config


__version__ = "1.0.0"


def run_cli():
    """Run BufferVault in CLI mode"""
    config = Config()
    monitor = ClipboardMonitor(config=config)
    
    print(f"BufferVault v{__version__}")
    print("=" * 50)
    print("Clipboard monitoring started in CLI mode")
    print("Press Ctrl+C to stop\n")
    
    monitor.start()
    
    try:
        # Keep running
        import time
        while True:
            time.sleep(1)
    except KeyboardInterrupt:
        print("\nStopping BufferVault...")
        monitor.stop()
        print("BufferVault stopped")


def show_history():
    """Show clipboard history in CLI"""
    config = Config()
    monitor = ClipboardMonitor(config=config)
    
    history = monitor.get_history(limit=20)
    
    print(f"BufferVault v{__version__} - Clipboard History")
    print("=" * 50)
    print(f"Total items in history: {len(history)}\n")
    
    for i, entry in enumerate(history):
        print(f"{i+1}. [{entry.get_display_time()}]")
        print(f"   {entry.get_preview(100)}")
        print()


def main():
    """Main entry point"""
    parser = argparse.ArgumentParser(
        description='BufferVault - Encrypted Clipboard History Manager'
    )
    parser.add_argument(
        '--mode',
        choices=['gui', 'cli', 'history'],
        default='gui',
        help='Run mode: gui (default), cli (background), or history (show history)'
    )
    parser.add_argument(
        '--version',
        action='version',
        version=f'BufferVault v{__version__}'
    )
    
    args = parser.parse_args()
    
    if args.mode == 'gui':
        gui_main()
    elif args.mode == 'cli':
        run_cli()
    elif args.mode == 'history':
        show_history()


if __name__ == '__main__':
    main()
