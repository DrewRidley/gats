
# Compiling and running 'TATs':

To compile and execute this program, rust must first be installed. It can be downloaded from:
    https://www.rust-lang.org/tools/install

On Windows, you must also install Visual Studio C++ (the rust installer should guide you through this).
Then, in your terminal, navigate to the 'tats' directory (root) and execute 'cargo run -- mariadb://user:password@localhost:3306/tats' with your credentials.
'tats' can be empty, or it can be initialized with the SQL schema provided.