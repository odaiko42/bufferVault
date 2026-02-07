"""
Basic tests for BufferVault components
"""
import unittest
import os
import shutil
import tempfile
from pathlib import Path

from config import Config
from encryption import EncryptionManager
from storage import StorageManager, ClipboardEntry


class TestConfig(unittest.TestCase):
    """Test configuration management"""
    
    def setUp(self):
        """Setup test environment"""
        self.test_dir = tempfile.mkdtemp()
        self.config_file = os.path.join(self.test_dir, 'test_config.json')
    
    def tearDown(self):
        """Cleanup test environment"""
        if os.path.exists(self.test_dir):
            shutil.rmtree(self.test_dir)
    
    def test_default_config(self):
        """Test default configuration"""
        config = Config(self.config_file)
        self.assertEqual(config.get('max_history_items'), 1000)
        self.assertTrue(config.get('encryption_enabled'))
    
    def test_save_and_load(self):
        """Test saving and loading configuration"""
        config = Config(self.config_file)
        config.set('max_history_items', 500)
        
        # Load again
        config2 = Config(self.config_file)
        self.assertEqual(config2.get('max_history_items'), 500)


class TestEncryption(unittest.TestCase):
    """Test encryption functionality"""
    
    def setUp(self):
        """Setup test environment"""
        self.test_dir = tempfile.mkdtemp()
        # Change to test dir to avoid polluting main directory
        self.original_dir = os.getcwd()
        os.chdir(self.test_dir)
        self.encryption = EncryptionManager(password=b"test_password")
    
    def tearDown(self):
        """Cleanup test environment"""
        os.chdir(self.original_dir)
        if os.path.exists(self.test_dir):
            shutil.rmtree(self.test_dir)
    
    def test_encrypt_decrypt_string(self):
        """Test encrypting and decrypting strings"""
        original = "Hello, World! This is a test message."
        encrypted = self.encryption.encrypt(original)
        decrypted = self.encryption.decrypt(encrypted)
        
        self.assertEqual(original, decrypted)
        self.assertNotEqual(original, encrypted)
    
    def test_encrypt_decrypt_unicode(self):
        """Test encrypting and decrypting unicode text"""
        original = "Hello 世界! 🎉 Testing unicode"
        encrypted = self.encryption.encrypt(original)
        decrypted = self.encryption.decrypt(encrypted)
        
        self.assertEqual(original, decrypted)
    
    def test_decrypt_wrong_data_fails(self):
        """Test that decrypting wrong data fails"""
        with self.assertRaises(ValueError):
            self.encryption.decrypt(b"invalid encrypted data")


class TestStorage(unittest.TestCase):
    """Test storage functionality"""
    
    def setUp(self):
        """Setup test environment"""
        self.test_dir = tempfile.mkdtemp()
        self.storage = StorageManager(
            storage_path=self.test_dir,
            encryption_enabled=False  # Disable for easier testing
        )
    
    def tearDown(self):
        """Cleanup test environment"""
        if os.path.exists(self.test_dir):
            shutil.rmtree(self.test_dir)
    
    def test_add_entry(self):
        """Test adding clipboard entry"""
        entry = self.storage.add_entry("Test content")
        self.assertIsNotNone(entry)
        self.assertEqual(entry.content, "Test content")
        
        history = self.storage.get_history()
        self.assertEqual(len(history), 1)
    
    def test_no_duplicate_entries(self):
        """Test that duplicate entries are not added"""
        self.storage.add_entry("Test content")
        result = self.storage.add_entry("Test content")
        
        self.assertIsNone(result)
        history = self.storage.get_history()
        self.assertEqual(len(history), 1)
    
    def test_multiple_entries(self):
        """Test adding multiple entries"""
        self.storage.add_entry("First")
        self.storage.add_entry("Second")
        self.storage.add_entry("Third")
        
        history = self.storage.get_history()
        self.assertEqual(len(history), 3)
        
        # Most recent should be first
        self.assertEqual(history[0].content, "Third")
        self.assertEqual(history[1].content, "Second")
        self.assertEqual(history[2].content, "First")
    
    def test_get_entry(self):
        """Test getting entry by index"""
        self.storage.add_entry("First")
        self.storage.add_entry("Second")
        
        entry = self.storage.get_entry(0)
        self.assertEqual(entry.content, "Second")
        
        entry = self.storage.get_entry(1)
        self.assertEqual(entry.content, "First")
    
    def test_search_history(self):
        """Test searching history"""
        self.storage.add_entry("Hello World")
        self.storage.add_entry("Python Programming")
        self.storage.add_entry("Hello Python")
        
        results = self.storage.search_history("python")
        self.assertEqual(len(results), 2)
        
        results = self.storage.search_history("hello")
        self.assertEqual(len(results), 2)
        
        results = self.storage.search_history("world")
        self.assertEqual(len(results), 1)
    
    def test_clear_history(self):
        """Test clearing history"""
        self.storage.add_entry("First")
        self.storage.add_entry("Second")
        
        self.storage.clear_history()
        history = self.storage.get_history()
        self.assertEqual(len(history), 0)
    
    def test_remove_entry(self):
        """Test removing specific entry"""
        self.storage.add_entry("First")
        self.storage.add_entry("Second")
        self.storage.add_entry("Third")
        
        result = self.storage.remove_entry(1)
        self.assertTrue(result)
        
        history = self.storage.get_history()
        self.assertEqual(len(history), 2)
        self.assertEqual(history[0].content, "Third")
        self.assertEqual(history[1].content, "First")


class TestClipboardEntry(unittest.TestCase):
    """Test clipboard entry functionality"""
    
    def test_entry_creation(self):
        """Test creating clipboard entry"""
        entry = ClipboardEntry("Test content")
        self.assertEqual(entry.content, "Test content")
        self.assertEqual(entry.entry_type, "text")
    
    def test_entry_preview(self):
        """Test entry preview"""
        short_content = "Short"
        entry = ClipboardEntry(short_content)
        self.assertEqual(entry.get_preview(), short_content)
        
        long_content = "A" * 150
        entry = ClipboardEntry(long_content)
        preview = entry.get_preview(100)
        self.assertEqual(len(preview), 103)  # 100 + "..."
        self.assertTrue(preview.endswith("..."))
    
    def test_entry_to_dict(self):
        """Test converting entry to dictionary"""
        entry = ClipboardEntry("Test", entry_type="text")
        data = entry.to_dict()
        
        self.assertEqual(data['content'], "Test")
        self.assertEqual(data['entry_type'], "text")
        self.assertIn('timestamp', data)
    
    def test_entry_from_dict(self):
        """Test creating entry from dictionary"""
        data = {
            'content': "Test content",
            'timestamp': 1234567890,
            'entry_type': 'text',
            'metadata': {}
        }
        
        entry = ClipboardEntry.from_dict(data)
        self.assertEqual(entry.content, "Test content")
        self.assertEqual(entry.timestamp, 1234567890)


if __name__ == '__main__':
    unittest.main()
