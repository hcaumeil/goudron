# goudron

Lightweight api tester.  
Write small scripts to automate test requests.  

**IMPORTANT! THIS PROJECT IS A WORK IN PROGRESS! ANYTHING CAN CHANGE AT ANY MOMENT WITHOUT ANY NOTICE! USE THIS LANGUAGE AT YOUR OWN RISK!**  

## Installation :

    $ git clone https://github.com/hcaumeil/goudron.git
    $ cd goudron
    $ cargo build --release
    # ln -s `readlink -f target/release/goudron` /usr/local/bin/goudron

## Uninstalling :
    $ rm -rf goudron
    # rm /usr/local/bin/goudron

## Quick start :

Create a file named hello.goud and write your first script : 

```
print "Hello, World!"
```

And run it : 

    $ goudron hello.goud

For more information about the goudron command, run : 
    
    $ goudron  -h

## Language Reference

### String

A string is any sequence of UTF-8 character between two `"`
You can escape only these things for now :
- `\n` - new line
- `\t` - tabulation
- `\\` - back slash
- `\"` - double quote

### Variable 

Define a variable to store a string in the memory.
Variable names are only made of ascii characters.
A value is expected to initialize a variable.
You can define a variable like this :

```
variableName = "value"
```

A variable can also be a copy of another variable :

```
otherVariable = "something"
variableName = otherVariable
```

Whenever a value is expected, you can sum strings and variables :

```
otherVariable = "Hello" + ", "
variableName = otherVariable + "World" + "!"
```

### Print

Print any value with the keyword print :

```
fox = "fox"
variable = fox + " jumps over the lazy dog"
print "the quick"
print "brown" + variable
```

### Request 

4 requests method are available :
- get
- post
- put
- delete

Write a simple request with a method name and an url : 

```
get "url" 
```

Every request verify the return code.
By default, it is 200, but you can specify it like this : 

```
get "url" 200
```

You can add a body with the request with the keyword body right after the url : 

```
post "url" body "value"
post "url" body "value" 200
```

Also, you can get the response into a variable like this : 


```
get "url" = variable
get "url" 200 = variable
post "url" body "value" 200 = variable
```

Or verify its value like this :

```
get "url" ? "value"
get "url" 200 ? variable
post "url" body "value" 200 ? "value" + variable
```
