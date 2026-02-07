"""
Encryption module for BufferVault
Handles encryption and decryption of clipboard data using AES-256
"""
from cryptography.fernet import Fernet
from cryptography.hazmat.primitives import hashes
from cryptography.hazmat.primitives.kdf.pbkdf2 import PBKDF2
from cryptography.hazmat.backends import default_backend
import base64
import os


class EncryptionManager:
    """Manages encryption and decryption of clipboard data"""
    
    def __init__(self, password=None):
        """
        Initialize encryption manager
        
        Args:
            password: Optional password for encryption. If None, a default is used.
        """
        self.password = password or self._get_default_password()
        self.salt = self._get_or_create_salt()
        self.key = self._derive_key(self.password, self.salt)
        self.cipher = Fernet(self.key)
    
    def _get_default_password(self):
        """Get default password from environment or generate one"""
        # In production, this should be user-provided
        # For now, use a machine-specific identifier
        import platform
        return f"BufferVault-{platform.node()}".encode()
    
    def _get_or_create_salt(self):
        """Get existing salt or create new one"""
        salt_file = '.vault_salt'
        if os.path.exists(salt_file):
            with open(salt_file, 'rb') as f:
                return f.read()
        else:
            salt = os.urandom(16)
            with open(salt_file, 'wb') as f:
                f.write(salt)
            return salt
    
    def _derive_key(self, password, salt):
        """Derive encryption key from password using PBKDF2"""
        if isinstance(password, str):
            password = password.encode()
        
        kdf = PBKDF2(
            algorithm=hashes.SHA256(),
            length=32,
            salt=salt,
            iterations=100000,
            backend=default_backend()
        )
        key = base64.urlsafe_b64encode(kdf.derive(password))
        return key
    
    def encrypt(self, data):
        """
        Encrypt data
        
        Args:
            data: Data to encrypt (string or bytes)
            
        Returns:
            Encrypted data as bytes
        """
        if isinstance(data, str):
            data = data.encode('utf-8')
        
        return self.cipher.encrypt(data)
    
    def decrypt(self, encrypted_data):
        """
        Decrypt data
        
        Args:
            encrypted_data: Encrypted data as bytes
            
        Returns:
            Decrypted data as string
        """
        try:
            decrypted = self.cipher.decrypt(encrypted_data)
            return decrypted.decode('utf-8')
        except Exception as e:
            raise ValueError(f"Decryption failed: {e}")
    
    def encrypt_file(self, input_path, output_path):
        """Encrypt a file"""
        with open(input_path, 'rb') as f:
            data = f.read()
        
        encrypted = self.encrypt(data)
        
        with open(output_path, 'wb') as f:
            f.write(encrypted)
    
    def decrypt_file(self, input_path, output_path):
        """Decrypt a file"""
        with open(input_path, 'rb') as f:
            encrypted_data = f.read()
        
        decrypted = self.decrypt(encrypted_data)
        
        with open(output_path, 'w', encoding='utf-8') as f:
            f.write(decrypted)
