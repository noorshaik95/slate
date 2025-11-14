#!/usr/bin/env python3
"""
Load test script to create 10k users and test rate limiting.

This script:
1. Creates 10,000 users via the API Gateway
2. Tests rate limiting on registration endpoint
3. Tests rate limiting on login endpoint
4. Provides detailed statistics and results
"""

import grpc
import sys
import time
import random
import string
from concurrent.futures import ThreadPoolExecutor, as_completed
from collections import defaultdict
from datetime import datetime

# Add proto path
sys.path.insert(0, '../proto')

# Import generated protobuf code
try:
    from user_pb2 import RegisterRequest, LoginRequest
    from user_pb2_grpc import UserServiceStub
except ImportError:
    print("Error: Could not import protobuf files.")
    print("Please ensure proto files are generated.")
    print("Run: cd proto && python -m grpc_tools.protoc -I. --python_out=. --grpc_python_out=. user.proto")
    sys.exit(1)


class UserLoadTester:
    def __init__(self, grpc_host="localhost", grpc_port=50051, num_users=10000):
        self.grpc_host = grpc_host
        self.grpc_port = grpc_port
        self.num_users = num_users
        self.channel = None
        self.stub = None
        self.stats = {
            'register_success': 0,
            'register_failed': 0,
            'register_rate_limited': 0,
            'login_success': 0,
            'login_failed': 0,
            'login_rate_limited': 0,
            'total_time': 0
        }
        self.created_users = []
        
    def connect(self):
        """Establish gRPC connection"""
        self.channel = grpc.insecure_channel(f'{self.grpc_host}:{self.grpc_port}')
        self.stub = UserServiceStub(self.channel)
        print(f"✓ Connected to gRPC server at {self.grpc_host}:{self.grpc_port}")
    
    def disconnect(self):
        """Close gRPC connection"""
        if self.channel:
            self.channel.close()
            print("✓ Disconnected from gRPC server")
    
    def generate_random_email(self, index):
        """Generate a unique email address"""
        random_suffix = ''.join(random.choices(string.ascii_lowercase + string.digits, k=6))
        return f"user{index}_{random_suffix}@loadtest.com"
    
    def generate_random_password(self):
        """Generate a random password"""
        return ''.join(random.choices(string.ascii_letters + string.digits + "!@#$%", k=12))
    
    def register_user(self, index):
        """Register a single user"""
        email = self.generate_random_email(index)
        password = self.generate_random_password()
        username = f"user{index}"
        
        try:
            request = RegisterRequest(
                email=email,
                password=password,
                username=username
            )
            response = self.stub.Register(request, timeout=10)
            self.stats['register_success'] += 1
            return {
                'success': True,
                'email': email,
                'password': password,
                'username': username,
                'user_id': response.user.id
            }
        except grpc.RpcError as e:
            if e.code() == grpc.StatusCode.RESOURCE_EXHAUSTED:
                self.stats['register_rate_limited'] += 1
                return {'success': False, 'rate_limited': True, 'error': str(e)}
            else:
                self.stats['register_failed'] += 1
                return {'success': False, 'rate_limited': False, 'error': str(e)}
    
    def login_user(self, email, password):
        """Login a user"""
        try:
            request = LoginRequest(
                email=email,
                password=password
            )
            response = self.stub.Login(request, timeout=10)
            self.stats['login_success'] += 1
            return {'success': True, 'token': response.access_token}
        except grpc.RpcError as e:
            if e.code() == grpc.StatusCode.RESOURCE_EXHAUSTED:
                self.stats['login_rate_limited'] += 1
                return {'success': False, 'rate_limited': True, 'error': str(e)}
            else:
                self.stats['login_failed'] += 1
                return {'success': False, 'rate_limited': False, 'error': str(e)}
    
    def create_users_batch(self, batch_size=100, max_workers=10):
        """Create users in batches with concurrent requests"""
        print(f"\n{'='*60}")
        print(f"Creating {self.num_users} users...")
        print(f"Batch size: {batch_size}, Workers: {max_workers}")
        print(f"{'='*60}\n")
        
        start_time = time.time()
        
        with ThreadPoolExecutor(max_workers=max_workers) as executor:
            futures = []
            for i in range(self.num_users):
                future = executor.submit(self.register_user, i)
                futures.append(future)
                
                # Progress update every 1000 users
                if (i + 1) % 1000 == 0:
                    print(f"Submitted {i + 1}/{self.num_users} registration requests...")
            
            # Collect results
            for i, future in enumerate(as_completed(futures)):
                result = future.result()
                if result['success']:
                    self.created_users.append(result)
                
                # Progress update
                if (i + 1) % 1000 == 0:
                    elapsed = time.time() - start_time
                    rate = (i + 1) / elapsed
                    print(f"Completed {i + 1}/{self.num_users} registrations "
                          f"({rate:.1f} req/s, {elapsed:.1f}s elapsed)")
        
        self.stats['total_time'] = time.time() - start_time
        
        print(f"\n{'='*60}")
        print(f"User Creation Complete!")
        print(f"{'='*60}")
        print(f"Total time: {self.stats['total_time']:.2f}s")
        print(f"Average rate: {self.num_users / self.stats['total_time']:.1f} req/s")
        print(f"Successful: {self.stats['register_success']}")
        print(f"Failed: {self.stats['register_failed']}")
        print(f"Rate Limited: {self.stats['register_rate_limited']}")
        print(f"{'='*60}\n")
    
    def test_rate_limiting_register(self, attempts=10):
        """Test rate limiting on registration endpoint"""
        print(f"\n{'='*60}")
        print(f"Testing Registration Rate Limiting")
        print(f"Attempting {attempts} rapid registrations from same IP...")
        print(f"Expected limit: 3 per hour")
        print(f"{'='*60}\n")
        
        rate_limited_count = 0
        success_count = 0
        
        for i in range(attempts):
            result = self.register_user(f"ratelimit_test_{i}")
            if result.get('rate_limited'):
                rate_limited_count += 1
                print(f"  [{i+1}] ❌ Rate limited (as expected after 3 attempts)")
            elif result['success']:
                success_count += 1
                print(f"  [{i+1}] ✓ Success")
            else:
                print(f"  [{i+1}] ✗ Failed: {result.get('error', 'Unknown error')}")
            
            time.sleep(0.1)  # Small delay between requests
        
        print(f"\nRate Limiting Test Results:")
        print(f"  Successful: {success_count}")
        print(f"  Rate Limited: {rate_limited_count}")
        print(f"  Expected behavior: First 3 succeed, rest rate limited")
        
        if rate_limited_count > 0:
            print(f"  ✓ Rate limiting is WORKING")
        else:
            print(f"  ⚠ Rate limiting may not be working properly")
        print(f"{'='*60}\n")
    
    def test_rate_limiting_login(self, attempts=10):
        """Test rate limiting on login endpoint"""
        print(f"\n{'='*60}")
        print(f"Testing Login Rate Limiting")
        print(f"Attempting {attempts} rapid logins...")
        print(f"Expected limit: 5 per 15 minutes")
        print(f"{'='*60}\n")
        
        # Use a created user or create a test user
        if self.created_users:
            test_user = self.created_users[0]
        else:
            print("Creating test user for login rate limit test...")
            test_user = self.register_user(999999)
            if not test_user['success']:
                print("Failed to create test user. Skipping login rate limit test.")
                return
        
        rate_limited_count = 0
        success_count = 0
        failed_count = 0
        
        for i in range(attempts):
            result = self.login_user(test_user['email'], test_user['password'])
            if result.get('rate_limited'):
                rate_limited_count += 1
                print(f"  [{i+1}] ❌ Rate limited (as expected after 5 attempts)")
            elif result['success']:
                success_count += 1
                print(f"  [{i+1}] ✓ Success")
            else:
                failed_count += 1
                print(f"  [{i+1}] ✗ Failed: {result.get('error', 'Unknown error')}")
            
            time.sleep(0.1)  # Small delay between requests
        
        print(f"\nRate Limiting Test Results:")
        print(f"  Successful: {success_count}")
        print(f"  Rate Limited: {rate_limited_count}")
        print(f"  Failed: {failed_count}")
        print(f"  Expected behavior: First 5 succeed, rest rate limited")
        
        if rate_limited_count > 0:
            print(f"  ✓ Rate limiting is WORKING")
        else:
            print(f"  ⚠ Rate limiting may not be working properly")
        print(f"{'='*60}\n")
    
    def print_summary(self):
        """Print final summary"""
        print(f"\n{'='*60}")
        print(f"LOAD TEST SUMMARY")
        print(f"{'='*60}")
        print(f"Target Users: {self.num_users}")
        print(f"Total Time: {self.stats['total_time']:.2f}s")
        print(f"\nRegistration Stats:")
        print(f"  Success: {self.stats['register_success']}")
        print(f"  Failed: {self.stats['register_failed']}")
        print(f"  Rate Limited: {self.stats['register_rate_limited']}")
        print(f"\nLogin Stats:")
        print(f"  Success: {self.stats['login_success']}")
        print(f"  Failed: {self.stats['login_failed']}")
        print(f"  Rate Limited: {self.stats['login_rate_limited']}")
        print(f"\nPerformance:")
        if self.stats['total_time'] > 0:
            print(f"  Average throughput: {self.num_users / self.stats['total_time']:.1f} req/s")
        print(f"{'='*60}\n")


def main():
    """Main execution function"""
    print(f"\n{'='*60}")
    print(f"USER LOAD TEST & RATE LIMIT VERIFICATION")
    print(f"{'='*60}")
    print(f"Start time: {datetime.now().strftime('%Y-%m-%d %H:%M:%S')}")
    print(f"{'='*60}\n")
    
    # Configuration
    GRPC_HOST = "localhost"
    GRPC_PORT = 50051
    NUM_USERS = 10000
    BATCH_SIZE = 100
    MAX_WORKERS = 20
    
    # Create tester instance
    tester = UserLoadTester(
        grpc_host=GRPC_HOST,
        grpc_port=GRPC_PORT,
        num_users=NUM_USERS
    )
    
    try:
        # Connect to gRPC server
        tester.connect()
        
        # Test rate limiting first (with smaller numbers)
        print("\n🔍 Phase 1: Rate Limiting Verification\n")
        tester.test_rate_limiting_register(attempts=10)
        
        # Wait a bit before login test
        print("Waiting 2 seconds before login rate limit test...")
        time.sleep(2)
        
        tester.test_rate_limiting_login(attempts=10)
        
        # Create 10k users
        print("\n🚀 Phase 2: Bulk User Creation\n")
        user_input = input(f"Proceed with creating {NUM_USERS} users? (yes/no): ")
        if user_input.lower() in ['yes', 'y']:
            tester.create_users_batch(
                batch_size=BATCH_SIZE,
                max_workers=MAX_WORKERS
            )
        else:
            print("Skipping bulk user creation.")
        
        # Print summary
        tester.print_summary()
        
    except KeyboardInterrupt:
        print("\n\n⚠ Test interrupted by user")
        tester.print_summary()
    except Exception as e:
        print(f"\n❌ Error: {e}")
        import traceback
        traceback.print_exc()
    finally:
        # Disconnect
        tester.disconnect()
        print(f"\nEnd time: {datetime.now().strftime('%Y-%m-%d %H:%M:%S')}")


if __name__ == "__main__":
    main()
