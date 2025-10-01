#!/usr/bin/env bash

set -e  # Exit on error

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
MODULE_NAME="usersu"
MODULE_FILE="${MODULE_NAME}.ko"

# Print functions
print_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

print_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

print_header() {
    echo -e "${GREEN}================================${NC}"
    echo -e "${GREEN}  UserSU Kernel Module Builder${NC}"
    echo -e "${GREEN}================================${NC}"
    echo ""
}

# Detect build environment
detect_environment() {
    print_info "Detecting build environment..."
    
    if [ -n "$ANDROID_BUILD_TOP" ] || [ -n "$ANDROID_PRODUCT_OUT" ]; then
        BUILD_ENV="android"
        print_info "Android build environment detected"
    else
        BUILD_ENV="linux"
        print_info "Standard Linux environment detected"
    fi
}

# Check dependencies
check_dependencies() {
    print_info "Checking dependencies..."
    
    if [ "$BUILD_ENV" = "linux" ]; then
        # Check for kernel headers
        if [ ! -d "/lib/modules/$(uname -r)/build" ]; then
            print_error "Kernel headers not found!"
            print_info "Install with: sudo apt-get install linux-headers-$(uname -r)"
            exit 1
        fi
        
        # Check for build tools
        if ! command -v make &> /dev/null; then
            print_error "make not found!"
            print_info "Install with: sudo apt-get install build-essential"
            exit 1
        fi
    else
        # Android specific checks
        if [ -z "$KERNEL_DIR" ]; then
            print_error "KERNEL_DIR not set for Android build"
            print_info "Set it with: export KERNEL_DIR=/path/to/kernel"
            exit 1
        fi
        
        if [ ! -d "$KERNEL_DIR" ]; then
            print_error "Kernel directory not found: $KERNEL_DIR"
            exit 1
        fi
    fi
    
    print_success "All dependencies satisfied"
}

# Create Makefile
create_makefile() {
    print_info "Creating Makefile..."
    
    if [ "$BUILD_ENV" = "linux" ]; then
        cat > Makefile << 'EOF'
# UserSU Kernel Module Makefile (Linux)

obj-m += usersu.o

KERNEL_DIR ?= /lib/modules/$(shell uname -r)/build
PWD := $(shell pwd)

all:
	$(MAKE) -C $(KERNEL_DIR) M=$(PWD) modules

clean:
	$(MAKE) -C $(KERNEL_DIR) M=$(PWD) clean
	rm -f Module.symvers modules.order

install: all
	sudo insmod $(PWD)/usersu.ko
	sudo chmod 666 /dev/usersu
	@echo "Module installed. Device: /dev/usersu"

uninstall:
	sudo rmmod usersu || true

check:
	@if lsmod | grep -q usersu; then \
		echo "UserSU module is loaded"; \
	else \
		echo "UserSU module is NOT loaded"; \
	fi

.PHONY: all clean install uninstall check
EOF
    else
        # Android Makefile
        cat > Makefile << 'EOF'
# UserSU Kernel Module Makefile (Android)

obj-m += usersu.o

# Android kernel configuration
KERNEL_DIR ?= $(ANDROID_PRODUCT_OUT)/obj/KERNEL_OBJ
ARCH ?= arm64
CROSS_COMPILE ?= aarch64-linux-android-

# Detect architecture if not set
ifeq ($(ARCH),)
    ARCH := $(shell file $(KERNEL_DIR)/vmlinux 2>/dev/null | grep -q "ARM aarch64" && echo arm64 || echo arm)
endif

PWD := $(shell pwd)

all:
	$(MAKE) -C $(KERNEL_DIR) M=$(PWD) ARCH=$(ARCH) CROSS_COMPILE=$(CROSS_COMPILE) modules

clean:
	$(MAKE) -C $(KERNEL_DIR) M=$(PWD) ARCH=$(ARCH) CROSS_COMPILE=$(CROSS_COMPILE) clean
	rm -f Module.symvers modules.order

.PHONY: all clean
EOF
    fi
    
    print_success "Makefile created"
}

# Build the module
build_module() {
    print_info "Building kernel module..."
    
    if [ "$BUILD_ENV" = "android" ]; then
        # Set default architecture if not specified
        if [ -z "$ARCH" ]; then
            export ARCH=arm64
            print_info "Architecture not set, defaulting to arm64"
        fi
        
        # Set default cross compiler if not specified
        if [ -z "$CROSS_COMPILE" ]; then
            # Try to find Android NDK compiler
            if [ -n "$ANDROID_NDK" ]; then
                export CROSS_COMPILE="$ANDROID_NDK/toolchains/llvm/prebuilt/linux-x86_64/bin/aarch64-linux-android-"
            else
                print_warning "CROSS_COMPILE not set and NDK not found"
                print_info "Set it with: export CROSS_COMPILE=aarch64-linux-android-"
            fi
        fi
    fi
    
    # Clean previous build
    make clean 2>/dev/null || true
    
    # Build
    if make -j$(nproc); then
        print_success "Module built successfully: ${MODULE_FILE}"
    else
        print_error "Build failed!"
        exit 1
    fi
}

# Verify module
verify_module() {
    if [ -f "$MODULE_FILE" ]; then
        print_info "Module information:"
        modinfo "$MODULE_FILE" 2>/dev/null || file "$MODULE_FILE"
        
        MODULE_SIZE=$(stat -f%z "$MODULE_FILE" 2>/dev/null || stat -c%s "$MODULE_FILE")
        print_info "Module size: $(numfmt --to=iec-i --suffix=B $MODULE_SIZE 2>/dev/null || echo $MODULE_SIZE bytes)"
    else
        print_error "Module file not found: ${MODULE_FILE}"
        exit 1
    fi
}

# Install module (Linux only)
install_module() {
    if [ "$BUILD_ENV" != "linux" ]; then
        print_warning "Auto-install only available for Linux"
        print_info "For Android, use: adb push ${MODULE_FILE} /data/local/tmp/"
        return
    fi
    
    print_info "Installing module..."
    
    # Check if already loaded
    if lsmod | grep -q "^${MODULE_NAME} "; then
        print_warning "Module already loaded, unloading first..."
        sudo rmmod "$MODULE_NAME" 2>/dev/null || true
    fi
    
    # Install
    if sudo insmod "$MODULE_FILE"; then
        print_success "Module loaded successfully"
    else
        print_error "Failed to load module"
        exit 1
    fi
    
    # Set permissions
    sleep 1
    if [ -e "/dev/${MODULE_NAME}" ]; then
        sudo chmod 666 "/dev/${MODULE_NAME}"
        print_success "Device node created: /dev/${MODULE_NAME}"
    else
        print_warning "Device node not found. Check dmesg for errors."
    fi
    
    # Show status
    print_info "Module status:"
    lsmod | grep "$MODULE_NAME"
}

# Show Android instructions
show_android_instructions() {
    if [ "$BUILD_ENV" = "android" ]; then
        echo ""
        print_info "To install on Android device:"
        echo "  1. adb root"
        echo "  2. adb remount"
        echo "  3. adb push ${MODULE_FILE} /data/local/tmp/"
        echo "  4. adb shell"
        echo "  5. su"
        echo "  6. insmod /data/local/tmp/${MODULE_FILE}"
        echo "  7. chmod 666 /dev/${MODULE_NAME}"
        echo ""
        print_warning "Note: SELinux may block the module. Use 'setenforce 0' if needed."
    fi
}

# Usage information
show_usage() {
    echo "Usage: $0 [OPTIONS]"
    echo ""
    echo "Options:"
    echo "  -h, --help          Show this help message"
    echo "  -c, --clean         Clean build artifacts"
    echo "  -b, --build         Build the module (default)"
    echo "  -i, --install       Build and install (Linux only)"
    echo "  -u, --uninstall     Uninstall the module (Linux only)"
    echo "  -k, --check         Check if module is loaded"
    echo ""
    echo "Environment variables for Android:"
    echo "  KERNEL_DIR          Path to Android kernel source"
    echo "  ARCH                Target architecture (arm64, arm)"
    echo "  CROSS_COMPILE       Cross compiler prefix"
    echo ""
    echo "Examples:"
    echo "  $0                  # Build module"
    echo "  $0 -i               # Build and install"
    echo "  KERNEL_DIR=/path/to/kernel ARCH=arm64 $0  # Android build"
}

# Main script
main() {
    print_header
    
    # Parse arguments
    ACTION="build"
    
    while [[ $# -gt 0 ]]; do
        case $1 in
            -h|--help)
                show_usage
                exit 0
                ;;
            -c|--clean)
                ACTION="clean"
                shift
                ;;
            -b|--build)
                ACTION="build"
                shift
                ;;
            -i|--install)
                ACTION="install"
                shift
                ;;
            -u|--uninstall)
                ACTION="uninstall"
                shift
                ;;
            -k|--check)
                ACTION="check"
                shift
                ;;
            *)
                print_error "Unknown option: $1"
                show_usage
                exit 1
                ;;
        esac
    done
    
    # Execute action
    case $ACTION in
        clean)
            print_info "Cleaning build artifacts..."
            make clean 2>/dev/null || rm -f *.o *.ko *.mod* .*.cmd modules.order Module.symvers
            rm -rf .tmp_versions
            print_success "Clean complete"
            ;;
        check)
            if lsmod | grep -q "^${MODULE_NAME} "; then
                print_success "UserSU module is loaded"
                lsmod | grep "$MODULE_NAME"
            else
                print_warning "UserSU module is NOT loaded"
            fi
            ;;
        uninstall)
            print_info "Unloading module..."
            sudo rmmod "$MODULE_NAME" 2>/dev/null && print_success "Module unloaded" || print_warning "Module not loaded"
            ;;
        build)
            detect_environment
            check_dependencies
            create_makefile
            build_module
            verify_module
            show_android_instructions
            print_success "Build complete!"
            ;;
        install)
            detect_environment
            check_dependencies
            create_makefile
            build_module
            verify_module
            install_module
            show_android_instructions
            print_success "Installation complete!"
            ;;
    esac
}

# Run main
main "$@"