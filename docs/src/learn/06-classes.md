# Classes

Create custom data types with `kin` (Scots for "family" or "type").

## Defining Classes

```scots
kin Person {
    dae init(name, age) {
        masel.name = name
        masel.age = age
    }

    dae greet() {
        blether f"Hullo, I'm {masel.name}!"
    }
}
```

### Creating Instances

```scots
ken hamish = Person("Hamish", 30)
ken morag = Person("Morag", 28)

blether hamish.name  # "Hamish"
blether morag.age    # 28

hamish.greet()  # "Hullo, I'm Hamish!"
```

## The init Method

The `init` method is the constructor - it runs when you create an instance:

```scots
kin Rectangle {
    dae init(width, height) {
        masel.width = width
        masel.height = height
        masel.area = width * height  # Computed on creation
    }
}

ken rect = Rectangle(5, 3)
blether rect.area  # 15
```

### Default Parameters in Constructors

```scots
kin Circle {
    dae init(radius = 1) {
        masel.radius = radius
    }

    dae area() {
        gie PI * masel.radius * masel.radius
    }
}

ken default_circle = Circle()
ken big_circle = Circle(10)

blether default_circle.radius  # 1
blether big_circle.radius      # 10
```

## Methods

Methods are functions defined inside a class:

```scots
kin BankAccount {
    dae init(owner, balance = 0) {
        masel.owner = owner
        masel.balance = balance
    }

    dae deposit(amount) {
        masel.balance = masel.balance + amount
        blether f"Deposited {amount}. New balance: {masel.balance}"
    }

    dae withdraw(amount) {
        gin amount > masel.balance {
            blether "Insufficient funds!"
            gie nae
        }
        masel.balance = masel.balance - amount
        blether f"Withdrew {amount}. New balance: {masel.balance}"
        gie aye
    }

    dae get_balance() {
        gie masel.balance
    }
}

ken account = BankAccount("Hamish", 100)
account.deposit(50)    # "Deposited 50. New balance: 150"
account.withdraw(30)   # "Withdrew 30. New balance: 120"
blether account.get_balance()  # 120
```

## The masel Keyword

`masel` (Scots for "myself") refers to the current instance:

```scots
kin Counter {
    dae init() {
        masel.count = 0
    }

    dae increment() {
        masel.count = masel.count + 1
        gie masel  # Return self for chaining
    }

    dae get() {
        gie masel.count
    }
}

ken c = Counter()
c.increment().increment().increment()
blether c.get()  # 3
```

## Inheritance with fae

Use `fae` (Scots for "from") to inherit from another class:

```scots
kin Animal {
    dae init(name) {
        masel.name = name
    }

    dae speak() {
        blether f"{masel.name} makes a noise"
    }

    dae describe() {
        blether f"This is an animal called {masel.name}"
    }
}

kin Dog fae Animal {
    dae init(name, breed) {
        masel.name = name
        masel.breed = breed
    }

    dae speak() {
        blether f"{masel.name} says: Woof!"
    }

    dae fetch() {
        blether f"{masel.name} fetches the ball!"
    }
}
```

### Using Inherited Classes

```scots
ken generic = Animal("Beastie")
generic.speak()     # "Beastie makes a noise"
generic.describe()  # "This is an animal called Beastie"

ken rex = Dog("Rex", "Labrador")
rex.speak()     # "Rex says: Woof!" (overridden)
rex.describe()  # "This is an animal called Rex" (inherited)
rex.fetch()     # "Rex fetches the ball!" (new method)

blether rex.breed  # "Labrador" (new attribute)
```

### Inheritance Hierarchy

```scots
kin Vehicle {
    dae init(brand) {
        masel.brand = brand
    }

    dae start() {
        blether "Starting..."
    }
}

kin Car fae Vehicle {
    dae init(brand, doors) {
        masel.brand = brand
        masel.doors = doors
    }

    dae honk() {
        blether "Beep beep!"
    }
}

kin ElectricCar fae Car {
    dae init(brand, doors, range) {
        masel.brand = brand
        masel.doors = doors
        masel.range = range
    }

    dae charge() {
        blether "Charging..."
    }
}

ken tesla = ElectricCar("Tesla", 4, 300)
tesla.start()   # Inherited from Vehicle
tesla.honk()    # Inherited from Car
tesla.charge()  # Own method
```

## Operator Overloading

Define special methods to customize how operators work with your class:

### Arithmetic Operators

```scots
kin Vector {
    dae init(x, y) {
        masel.x = x
        masel.y = y
    }

    # + operator
    dae __pit_thegither__(that) {
        gie Vector(masel.x + that.x, masel.y + that.y)
    }

    # - operator
    dae __tak_awa__(that) {
        gie Vector(masel.x - that.x, masel.y - that.y)
    }

    # * operator
    dae __times__(scalar) {
        gie Vector(masel.x * scalar, masel.y * scalar)
    }
}

ken v1 = Vector(3, 4)
ken v2 = Vector(1, 2)

ken sum = v1 + v2      # Vector(4, 6)
ken diff = v1 - v2     # Vector(2, 2)
ken scaled = v1 * 3    # Vector(9, 12)
```

### Comparison Operators

```scots
kin Money {
    dae init(amount) {
        masel.amount = amount
    }

    dae __same_as__(that) {
        gie masel.amount == that.amount
    }

    dae __wee_er__(that) {
        gie masel.amount < that.amount
    }

    dae __muckle_er__(that) {
        gie masel.amount > that.amount
    }
}

ken price1 = Money(10)
ken price2 = Money(20)

blether price1 == price1  # aye
blether price1 < price2   # aye
blether price2 > price1   # aye
```

### All Overloadable Operators

| Method | Operator | Scots Meaning |
|--------|----------|---------------|
| `__pit_thegither__` | `+` | Put together |
| `__tak_awa__` | `-` | Take away |
| `__times__` | `*` | Multiply |
| `__pairt__` | `/` | Divide |
| `__lave__` | `%` | Remainder |
| `__same_as__` | `==` | Same as |
| `__differs_fae__` | `!=` | Differs from |
| `__wee_er__` | `<` | Smaller |
| `__wee_er_or_same__` | `<=` | Smaller or same |
| `__muckle_er__` | `>` | Bigger |
| `__muckle_er_or_same__` | `>=` | Bigger or same |

## Practical Examples

### Stack Class

```scots
kin Stack {
    dae init() {
        masel.items = []
    }

    dae push(item) {
        shove(masel.items, item)
    }

    dae pop() {
        gin masel.is_empty() {
            gie naething
        }
        gie yank(masel.items)
    }

    dae peek() {
        gin masel.is_empty() {
            gie naething
        }
        gie bum(masel.items)
    }

    dae is_empty() {
        gie len(masel.items) == 0
    }

    dae size() {
        gie len(masel.items)
    }
}

ken stack = Stack()
stack.push(1)
stack.push(2)
stack.push(3)

blether stack.peek()  # 3
blether stack.pop()   # 3
blether stack.size()  # 2
```

### Linked List

```scots
kin Node {
    dae init(value) {
        masel.value = value
        masel.next = naething
    }
}

kin LinkedList {
    dae init() {
        masel.head = naething
        masel.length = 0
    }

    dae append(value) {
        ken new_node = Node(value)
        gin masel.head == naething {
            masel.head = new_node
        } ither {
            ken current = masel.head
            whiles current.next != naething {
                current = current.next
            }
            current.next = new_node
        }
        masel.length = masel.length + 1
    }

    dae to_list() {
        ken result = []
        ken current = masel.head
        whiles current != naething {
            shove(result, current.value)
            current = current.next
        }
        gie result
    }
}

ken list = LinkedList()
list.append(1)
list.append(2)
list.append(3)
blether list.to_list()  # [1, 2, 3]
```

### Game Entity

```scots
kin Entity {
    dae init(name, x, y) {
        masel.name = name
        masel.x = x
        masel.y = y
        masel.health = 100
    }

    dae move(dx, dy) {
        masel.x = masel.x + dx
        masel.y = masel.y + dy
    }

    dae take_damage(amount) {
        masel.health = max(0, masel.health - amount)
        gin masel.health == 0 {
            blether f"{masel.name} has been defeated!"
        }
    }

    dae is_alive() {
        gie masel.health > 0
    }
}

kin Player fae Entity {
    dae init(name) {
        masel.name = name
        masel.x = 0
        masel.y = 0
        masel.health = 100
        masel.score = 0
    }

    dae collect_coin() {
        masel.score = masel.score + 10
    }
}

ken player = Player("Hamish")
player.move(5, 3)
player.collect_coin()
blether f"Score: {player.score}"  # "Score: 10"
```

## Exercises

1. **Rectangle Class**: Create a Rectangle class with width, height, and methods for area and perimeter

2. **Fraction Class**: Create a Fraction class with operator overloading for arithmetic

3. **Todo List**: Create a TodoItem and TodoList class

<details>
<summary>Solutions</summary>

```scots
# 1. Rectangle Class
kin Rectangle {
    dae init(width, height) {
        masel.width = width
        masel.height = height
    }

    dae area() {
        gie masel.width * masel.height
    }

    dae perimeter() {
        gie 2 * (masel.width + masel.height)
    }

    dae is_square() {
        gie masel.width == masel.height
    }
}

ken rect = Rectangle(4, 5)
blether rect.area()       # 20
blether rect.perimeter()  # 18

# 2. Fraction Class
kin Fraction {
    dae init(num, den) {
        masel.num = num
        masel.den = den
    }

    dae __pit_thegither__(that) {
        ken new_num = masel.num * that.den + that.num * masel.den
        ken new_den = masel.den * that.den
        gie Fraction(new_num, new_den)
    }

    dae __times__(that) {
        gie Fraction(masel.num * that.num, masel.den * that.den)
    }

    dae __same_as__(that) {
        gie masel.num * that.den == that.num * masel.den
    }

    dae to_string() {
        gie f"{masel.num}/{masel.den}"
    }
}

ken half = Fraction(1, 2)
ken third = Fraction(1, 3)
ken sum = half + third
blether sum.to_string()  # "5/6"

# 3. Todo List
kin TodoItem {
    dae init(text) {
        masel.text = text
        masel.done = nae
    }

    dae complete() {
        masel.done = aye
    }
}

kin TodoList {
    dae init() {
        masel.items = []
    }

    dae add(text) {
        shove(masel.items, TodoItem(text))
    }

    dae complete(index) {
        gin index >= 0 an index < len(masel.items) {
            masel.items[index].complete()
        }
    }

    dae show() {
        fer i in 0..len(masel.items) {
            ken item = masel.items[i]
            ken status = gin item.done than "[x]" ither "[ ]"
            blether f"{i}. {status} {item.text}"
        }
    }
}

ken todos = TodoList()
todos.add("Learn mdhavers")
todos.add("Build something braw")
todos.complete(0)
todos.show()
```

</details>

## Next Steps

Learn about [modules](./07-modules.md) to organize larger programs into multiple files.
