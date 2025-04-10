
# General Dependencies

Make sure you have the required hardware and software to run CUDA programs:

-   A CUDA-enabled GPU with [compute capability](https://developer.nvidia.com/cuda-gpus) >= 5
-   [cuda-toolkit](https://developer.nvidia.com/cuda-toolkit) version 12.x
-   An appropriate nvidia driver ([see table here](https://docs.nvidia.com/cuda/cuda-toolkit-release-notes/index.html#id6))
-   [Rustup](https://rustup.rs/)

# Downloading this compiler

Download the compiler from the [releases page](https://github.com/NiekAukes/rust/releases/) and unpack it.
(or directly download for [windows](http://aukespot.com/rust-gpuhc/rust-gpuhc-windows.zip) or [linux](http://aukespot.com/rust-gpuhc/rust-gpuhc-linux.zip))

continue with [Linking the compiler and cloning the sample project](#using-this-compiler-with-the-sample-project)

# Installing this compiler from source

**Note: This document is modified from INSTALL.md and describes _building_ Rust _from source_.
**

Make sure you have installed the following build dependencies:

-   `python` 3 or 2.7
-   `git`
-   A C compiler (when building for the host, `cc` is enough; cross-compiling may
    need additional compilers)
-   `curl` (not needed on Windows)
-   `pkg-config` if you are compiling on Linux and targeting Linux
-   `libiconv` (already included with glibc on Debian-based distros)

To build Cargo, you'll also need OpenSSL (`libssl-dev` or `openssl-devel` on
most Unix distros).

On this compiler version, you'll need additional tools to compile LLVM:

-   `g++`, `clang++`, or MSVC with versions listed on
    [LLVM's documentation](https://llvm.org/docs/GettingStarted.html#host-c-toolchain-both-compiler-and-standard-library)
-   `ninja`, or GNU `make` 3.81 or later (Ninja is recommended, especially on
    Windows)
-   `cmake` 3.13.4 or later
-   `libstdc++-static` may be required on some Linux distributions such as Fedora
    and Ubuntu

## Building on a Unix-like system

### Build steps

1. Clone the [source] with `git`:

    ```sh
    git clone https://github.com/NiekAukes/rust.git
    cd rust
    git checkout kernel-dev-codegen
    ```

[source]: https://github.com/NiekAukes/rust

2. Configure the build settings:

    ```sh
    ./configure --set build.extended=false --set rust.deny-warnings=false
    ```

3. Build:

    ```sh
    ./x.py build --stage 1
    ```

## Building on Windows

### MSVC

MSVC builds of Rust additionally require an installation of Visual Studio 2017
(or later) so `rustc` can use its linker. The simplest way is to get
[Visual Studio], check the "C++ build tools" and "Windows 10 SDK" workload.

[Visual Studio]: https://visualstudio.microsoft.com/downloads/ (If you're installing CMake yourself, be careful that "C++ CMake tools for
Windows" doesn't get included under "Individual components".)

With these dependencies installed, you can build the compiler in a `cmd.exe`
shell by:

1. Clone the [source] with `git`:

    ```batch
    git clone https://github.com/NiekAukes/rust.git
    cd rust
    git checkout kernel-dev-codegen
    ```

2. Configure the build settings:

    ```batch
    x setup compiler
    ```

3. Build:

    ```sh
    x build --stage 1 --set llvm.download-ci-llvm=false --set rust.deny-warnings=false
    ```

Right now, building Rust only works with some known versions of Visual Studio.
If you have a more recent version installed and the build system doesn't
understand, you may need to force rustbuild to use an older version.
This can be done by manually calling the appropriate vcvars file before running
the bootstrap.

```batch
CALL "C:\Program Files (x86)\Microsoft Visual Studio\2019\Community\VC\Auxiliary\Build\vcvars64.bat"
python x.py build
```

# Using this compiler with the sample project

## Clone the sample project including dependencies

you can clone the sample project setup found on the [rust-kernels](https://github.com/NiekAukes/rust-kernels) repository, Preferably outside the compiler folder to avoid confusion.
In rust-kernels, there is the sample folder, and 3 dependencies to execute the code on the GPU.

## Linking the compiler

There are 2 methods to install the compiler.

### 1. Link a new toolchain via rustup (recommended):

```sh
rustup toolchain link rust-gpuhc [path-to-compiler]/rust-gpuhc
```

or when building from source:

```sh
rustup toolchain link rust-gpuhc [path-to-compiler]/build/host/stage1
```

<!-- Then in the sample folder, create a new file called `rust-toolchain` (without file extension), and paste in `rust-gpuhc`. Cargo should now know to compile the project with the modified compiler -->

### 2. Link via cargo:

in the sample project, create a new folder called `.cargo` and in that folder a file `config.toml`. This file should have contents similar to
`[build]
    rustc = "[path-to-compiler]/rust-gpuhc/bin/rustc"
    // or when building from source
    rustc = "[path-to-compiler]/build/host/stage1/bin/rustc"`

please note that rust-analyzer is not used to this compiler and may give faulty feedback. please refer to the compiler output for potential syntax errors.

## Writing code

<!-- see the README.md in the [rust-kernels](https://github.com/NiekAukes/rust-kernels) repository for more information on how to run and write code with this compiler. -->

### Building and running with cargo

To build and run the code, you can use the `cargo` tool Rust provides. Make sure you have followed the steps above to link the compiler to rustup or cargo.

### Defining a kernel

With this compiler, writing code for the GPU is quite straightforward. To designate a function to be runnable on the GPU, use the `#[kernel]` attribute. This attribute can only be used on functions. Furthermore, this function is not callable anymore on the CPU as it is entirely replaced by a bytecode reference.

```rust
// an example of a kernel function that fills a simple array
#[kernel]
unsafe fn gpu64(mut a: Buffer<i32>) {
    let i = gpu::global_tid_x();
    a.set(i as usize, i);
}
```

### Mutable buffers

An important conceptual limitation of GPU programming in Rust is that mutable references may not be passed to the GPU. This is because the reference is inherently copied to multiple threads, which is not allowed in Rust. To work around this, the compiler provides a `Buffer<T>` type, which refers to a mutable buffer on the GPU. This buffer can be read from and written to without any issues using the `set` and `get` methods.

```rust
#[kernel]
unsafe fn add(mut a: Buffer<i32>,
              mut b: Buffer<i32>,
              mut out: Buffer<i32>) {
    let i = gpu::global_tid_x() as usize;
    out.set(i, a.get(i) + b.get(i));
    a.set(i, 0);
    b.set(i, 0);
}
```

This constraint is not directly enforced by the compiler. However, the interface to run GPU code only accepts `Buffer<T>` types, and will not allow you to pass mutable references.

### Supported language features

Unfortunately not all language features are supported by the compiler, and some care should be taken when writing code. A very stringent limitation is that the compiler cannot use features defined in the `std` library, even if they can be defined for non-std use cases. Examples of this are panics and (dynamic) memory allocation. The compiler does not support these features, and will crash when you use them.

### Specifying an engine

To make a program compilable, an engine must be specified. This is done by adding the `#![engine(cuda::engine)]` attribute to the crate root. This attribute is required for the compiler to know where to store the compiled code.

<!-- For devices that don't support CUDA, the `#![engine(placeholder)]` attribute can be used. This engine will compile the code, but won't provide any functionality to run it. -->

## Running code

> Note: This section assumes you are using the cuda engine as specified in the previous section.

To run a gpu kernel, you can call the `kernel.launch(threads, blocks, args...)` function. This function takes the desired number of threads to run per block, the number of blocks to run, and the arguments to pass to the kernel. The arguments must be of the same type as the kernel arguments.

```rust
#[kernel]
unsafe fn add2(a: &[i32], b: &[i32], mut out: Buffer<i32>) {
    let i = gpu::global_tid_x() as usize;
    out.set(i, a[i] + b[i]);
}

fn main() {
    let a = vec![1, 2, 3, 4, 5];
    let b = vec![5, 4, 3, 2, 1];

    let out = Buffer::alloc(5).unwrap();

    add2.launch(5, 1, &a.as_slice(), &b.as_slice(), out)
        .unwrap();

    let result = out.retrieve().unwrap();
    println!("{:?}", result);
}
```

### Instantiating buffers

Unlike other types, `Buffer<T>` cannot be used on the CPU. Buffers created are only valid on the GPU. Using a buffer on the CPU will result in UB. `Buffer::alloc` is used to create a new empty buffer with memory allocated on the GPU. To create a buffer with data, use `Buffer::allocate_with`.

To copy data from the GPU, you can use the `retrieve` method. This method will copy the data from the GPU to the CPU and return it as a vector.

### Using launch_with_dptr

The `launch_with_dptr` function is a more advanced version of the `launch` function. It allows you to pass pointers to the device memory as an argument. This can be useful when you want to pass data that is continuously updated on the GPU. Use the `to_device` method to copy variables to the GPU.

```rust
fn main() {
    let a = vec![1, 2, 3, 4, 5];
    let b = vec![5, 4, 3, 2, 1];

    // with a Buffer, the data is already on the GPU
    // the to_device method simply converts the Buffer to a DPtr
    let mut out = Buffer::<i32>::alloc(5).unwrap().to_device().unwrap();

    let mut da = a.as_slice().to_device().unwrap();
    let mut db = b.as_slice().to_device().unwrap();

    add2.launch_with_dptr(5, 1, &mut da, &mut db, &mut out);

    let out = out.retrieve();
    println!("{:?}", out);
}
```

### Unforseen Errors

The compiler is still in development, and some features may not work as expected. This usually results in a crash of the compiler, but may also result in UB. If you encounter any issues, please report them on the [issues page](https://github.com/NiekAukes/rust/issues) of the compiler repository. Please include the code that caused the issue.

### Known Issues

-   incompatible NVVM version: Most likely, your driver version is not compatible with the CUDA toolkit you're running. Please install an appropriate nvidia driver ([see table here](https://docs.nvidia.com/cuda/cuda-toolkit-release-notes/index.html#id6))

-   "parse invalid cast opcode for cast from 'i8\*' to 'i64'": This is a known issue with the compiler. Compiling with --release should fix this issue in most cases. If not, please report it on the issues page.
