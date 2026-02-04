## tsi file

The `*.tsi` files, are DTS simulation trajectory outputs. It contains
information about vertices, triangles and inclusion positions.<br> It can be
used as input to the [PLM](#pointillism-plm) executable and it is formated as
shown below:

### General Structure

1. Each `.tsi` file begins with a line specifying **version 1.1**.
2. The next line defines the **box size** (`x`, `y`, and `z`) of the system in
   **nm**.
3. The subsequent three sections describe the **TS mesh**. Each section starts
   with a keyword (`vertex`, `triangle`, or `inclusion`) and their respective
   counts.

```
version 1.1
box 50.000     50.000     50.000
vertex 1840
0       21.4    33.8    32.7    0
1       38.1    26.1    32.3    0
2       40.9    24.2    19.9    0
...
1839    31.2   323.2    23      0
triangle 3680
0        75      776    1043    1
1       796     1821     752    1
2       995     1027     279    1
3       662    1162      56     1
4       167      38     391     1
...
inclusion 3
0         1       22       0    1
1         1        5       0    1
2         2       30       0    1
```

### Vertex section

- The file includes **1840 vertices**.
- Each vertex is assigned:
  - An **index**.
  - A **position** in `x`, `y`, and `z`.
  - An integer representing the **domain**.

### Triangles

- The 1840 vertices are connected via **3680 triangles**.
- Each triangle is defined by:
  - An **index**.
  - The **vertices** it connects.
  - An integer representing the **type**.
- Example:
  - Triangle `0` connects vertices `75`, `776`, and `1043`.

### Inclusions

- A `.tsi` file may include a section for **(protein) inclusions**.
- In this example:
  - There are **three inclusions** of **two different types**.
- Each inclusion is defined by:
  1. An **index**.
  2. The **inclusion type** (e.g., type `1` for inclusions `0` and `1`, type
     `2` for inclusion `2`).
  3. The **corresponding vertex index**.
  4. **Two floating-point numbers**:
     - These describe a unit two-dimensional vector.
     - The numbers sum to **1**.
     - They define the **orientation** of the inclusion relative to the bilayer
       normal.
