This file was sourced from [web.archive.org][archived_spec]

# 0. Foreword

This purpose of this page is to provide documentation for the DVI file format. 
Much of the content on this page was taken directly from the dvitype.web 
program, written by Donald Knuth (this explains the several references to the 
DVItype program; these can be ignored).

# 1. The DVI File Format

Before we get into the details of DVItype, we need to know exactly what DVI 
files are. The form of such files was designed by David R. Fuchs in 1979. 
Almost any reasonable typesetting device can be driven by a program that takes 
DVI files as input, and dozens of such DVI-to-whatever programs have been 
written. Thus, it is possible to print the output of document compilers like 
TeX on many different kinds of equipment.

A DVI file is a stream of 8-bit bytes, which may be regarded as a series of 
commands in a machine-like language. The first byte of each command is the 
operation code, and this code is followed by zero or more bytes that provide 
parameters to the command. The parameters themselves may consist of several 
consecutive bytes; for example, the set_rule command has two parameters, each 
of which is four bytes long. Parameters are usually regarded as nonnegative 
integers; but four-byte-long parameters, and shorter parameters that denote 
distances, can be either positive or negative. Such parameters are given in 
two's complement notation. For example, a two-byte-long distance parameter 
has a value between -2^15 and 2^15-1. [NOTE: DVI files use big endian format 
for multiple byte integer parameters.]

A DVI file consists of a "preamble," followed by a sequence of one or more 
"pages," followed by a "postamble." The preamble is simply a pre command, 
with its parameters that define the dimensions used in the file; this must 
come first. Each "page" consists of a bop command, followed by any number of 
other commands that tell where characters are to be placed on a physical page, 
followed by an eop command. The pages appear in the order that they were 
generated, not in any particular numerical order. If we ignore nop commands 
and fnt_def commands (which are allowed between any two commands in the file), 
each eop command is immediately followed by a bop command, or by a post 
command; in the latter case, there are no more pages in the file, and the 
remaining bytes form the postamble. Further details about the postamble will 
be explained later.

Some parameters in DVI commands are "pointers." These are four-byte quantities 
that give the location number of some other byte in the file; the first byte is 
number 0, then comes number 1, and so on. For example, one of the parameters of 
a bop command points to the previous bop; this makes it feasible to read the 
pages in backwards order, in case the results are being directed to a device 
that stacks its output face up. Suppose the preamble of a DVI file occupies 
bytes 0 to 99. Now if the first page occupies bytes 100 to 999, say, and if the 
second page occupies bytes 1000 to 1999, then the bop that starts in byte 1000 
points to 100 and the bop that starts in byte 2000 points to 1000. (The very 
first bop, i.e., the one that starts in byte 100, has a pointer of -1.)

The DVI format is intended to be both compact and easily interpreted by a 
machine. Compactness is achieved by making most of the information implicit 
instead of explicit. When a DVI-reading program reads the commands for a page, 
it keeps track of several quantities: (a) The current font f is an integer; 
this value is changed only by fnt and fnt_num commands. (b) The current 
position on the page is given by two numbers called the horizontal and vertical 
coordinates, h and v. Both coordinates are zero at the upper left corner of the 
page; moving to the right corresponds to increasing the horizontal coordinate, 
and moving down corresponds to increasing the vertical coordinate. Thus, the 
coordinates are essentially Cartesian, except that vertical directions are 
flipped; the Cartesian version of (h,v) would be (h,-v). (c) The current 
spacing amounts are given by four numbers w, x, y, and z, where w and x are 
used for horizontal spacing and where y and z are used for vertical spacing. 
(d) There is a stack containing (h,v,w,x,y,z) values; the DVI commands push 
and pop are used to change the current level of operation. Note that the 
current font f is not pushed and popped; the stack contains only information 
about positioning.

The values of h, v, w, x, y, and z are signed integers having up to 32 bits, 
including the sign. Since they represent physical distances, there is a small 
unit of measurement such that increasing h by 1 means moving a certain tiny 
distance to the right. The actual unit of measurement is variable, as explained 
below.

# 3. Table of Opcodes

The following table gives the instruction set for DVI. The parameters are 
listed in the order they would appear in a DVI file; the number in brackets 
gives the size of the parameter (in bytes).

The DVI Instruction Set

Opcode  | Instruction Name | Parameters | Description
--------|------------------|------------|------------
0...127 | set_char_i       |            | typeset a character and move right
128     | set1             | c[1]       | typeset a character and move right
129     | set2             | c[2]       | "
130     | set3             | c[3]       | "
131     | set4             | c[4]       | "
132     | set_rule         | a[4], b[4] | typeset a rule and move right
133     | put1             | c[1]       | typeset a character
134     | put2             | c[2]       | "
135     | put3             | c[3]       | "
136     | put4             | c[4]       | "
137     | put_rule         | a[4], b[4] | typeset a rule
138     | nop              |            | no operation
139     | bop              | c_0[4]..c_9[4], p[4] | beginning of page
140     | eop              |            | ending of page
141     | push             |            | save the current positions
142     | pop              |            | restore previous positions
143     | right1           | b[1]       | move right
144     | right2           | b[2]       | "
145     | right3           | b[3]       | "
146     | right4           | b[4]       | "
147     | w0               |            | move right by w
148     | w1               | b[1]       | move right and set w
149     | w2               | b[2]       | "
150     | w3               | b[3]       | "
151     | w4               | b[4]       | "
152     | x0               |            | move right by x
153     | x1               | b[1]       | move right and set x
154     | x2               | b[2]       | "
155     | x3               | b[3]       | "
156     | x4               | b[4]       | "
157     | down1            | a[1]       | move down
158     | down2            | a[2]       | "
159     | down3            | a[3]       | "
160     | down4            | a[4]       | "
161     | y0               |            | move down by y
162     | y1               | a[1]       | move down and set y
163     | y2               | a[2]       | " 
164     | y3               | a[3]       | "
165     | y4               | a[4]       | "
166     | z0               |            | move down by z
167     | z1               | a[1]       | move down and set z
168     | z2               | a[2]       | "
169     | z3               | a[3]       | "
170     | z4               | a[4]       | "
171...234 | fnt_num_i      |            | set current font to i
235     | fnt1             | k[1]       | set current font
236     | fnt2             | k[2]       | "
237     | fnt3             | k[3]       | "
238     | fnt4             | k[4]       | "
239     | xxx1             | k[1], x[k] | extension to DVI primitives
240     | xxx2             | k[2], x[k] | "
241     | xxx3             | k[3], x[k] | "
242     | xxx4             | k[4], x[k] | " 
243     | fnt_def1         | k[1], c[4], s[4], d[4], a[1], l[1], n[a+l] | define the meaning of a font number
244     | fnt_def2         | k[2], c[4], s[4], d[4], a[1], l[1], n[a+l] | "
245     | fnt_def3         | k[3], c[4], s[4], d[4], a[1], l[1], n[a+l] | "
246     | fnt_def4         | k[4], c[4], s[4], d[4], a[1], l[1], n[a+l] | "
247     | pre              | i[1], num[4], den[4], mag[4], k[1], x[k]   | preamble
248     | post             | p[4], num[4], den[4], mag[4], l[4], u[4], s[2], t[2] < font definitions > | postamble beginning
249     | post_post        | q[4], i[1]; 223's | postamble ending
250...255 | undefined      |            |

# 4. Description of Opcodes

 1. Opcodes 0-127: *set_char_i (0 <= i <= 127)*

    Typeset character number i from font f such that the reference point of the character is at (h,v). Then increase h by the width of that character. Note that a character may have zero or negative width, so one cannot be sure that h will advance after this command; but h usually does increase.

 2. Opcodes 128-131: *seti (1 <= i <= 4); c[i]*

    Same as set_char_0, except that character number c is typeset. TeX82 uses the set1 command for characters in the range *128 <= c < 256*. TeX82 never uses the set2, command which is intended for processors that deal with oriental languages; but DVItype will allow character codes greater than 255, assuming that they all have the same width as the character whose code is c mod 256.

 3. Opcode 132: *set_rule; a[4], b[4]*

    Typeset a solid black rectangle of height a and width b, with its bottom left corner at (h,v). Then set h:=h+b. If either *a <= 0* or *b <= 0*, nothing should be typeset. Note that if *b < 0*, the value of h will decrease even though nothing else happens. Programs that typeset from DVI files should be careful to make the rules line up carefully with digitized characters, as explained in connection with the rule_pixels subroutine below.

 4. Opcodes 133-136: *puti (1 <= i <= 4); c[i]*

    Typeset character number c from font f such that the reference point of the character is at (h,v). (The put commands are exactly like the set commands, except that they simply put out a character or a rule without moving the reference point afterwards.)

 5. Opcode 137: put_rule; a[4], b[4]

    Same as set_rule, except that h is not changed.

 6. Opcode 138: nop

    No operation, do nothing. Any number of nop's may occur between DVI commands, but a nop cannot be inserted between a command and its parameters or between two parameters.

 7. Opcode 139: bop; c_0[4]..c_9[4], p[4]

    Beginning of a page: Set (h,v,w,x,y,z):=(0,0,0,0,0,0) and set the stack empty. Set the current font f to an undefined value. The ten c_i parameters can be used to identify pages, if a user wants to print only part of a DVI file; TeX82 gives them the values of \count0...\count9 at the time \shipout was invoked for this page. The parameter p points to the previous bop command in the file, where the first bop has p=-1.

 8. Opcode 140: eop

    End of page: Print what you have read since the previous bop. At this point the stack should be empty. (The DVI-reading programs that drive most output devices will have kept a buffer of the material that appears on the page that has just ended. This material is largely, but not entirely, in order by v coordinate and (for fixed v) by h coordinate; so it usually needs to be sorted into some order that is appropriate for the device in question. DVItype does not do such sorting.)

 9. Opcode 141: push

    Push the current values of (h,v,w,x,y,z) onto the top of the stack; do not change any of these values. Note that f is not pushed.

 10. Opcode 142: pop

    Pop the top six values off of the stack and assign them to (h,v,w,x,y,z). The number of pops should never exceed the number of pushes, since it would be highly embarrassing if the stack were empty at the time of a pop command.
    
 11. Opcodes 143-146: *righti (1 <= i <= 4); b[i]*

    Set h:=h+b, i.e., move right b units. The parameter is a signed number in two's complement notation; if *b < 0*, the reference point actually moves left.

 12. Opcodes 147-151: *wi (0 <= i <= 4); b[i]*

    The w0 command sets h:=h+w; i.e., moves right w units. With luck, this parameterless command will usually suffice, because the same kind of motion will occur several times in succession. The other w commands set w:=b and h:=h+b. The value of b is a signed quantity in two's complement notation. This command changes the current w spacing and moves right by b.

 13. Opcodes 152-156: *xi (0 <= i <= 4); b[i]*

    The parameterless x0 command sets h:=h+x; i.e., moves right x units. The x commands are like the w commands except that they involve x instead of w. The other x commands set x:=b and h:=h+b. The value of b is a signed quantity in two's complement notation. This command changes the current x spacing and moves right by b.

 14. Opcodes 157-160: *downi (1 <= i <= 4); a[i]*

    Set v:=v+a, i.e., move down a units. The parameter is a signed number in two's complement notation; if *a < 0*, the reference point actually moves up.

 15. Opcodes 161-165: *yi (0 <= i <= 4); a[i]*

    The y0 command sets v:=v+y; i.e., moves down y units. With luck, this parameterless command will usually suffice, because the same kind of motion will occur several times in succession. The other y commands set y:=a and v:=v+a. The value of a is a signed quantity in two's complement notation. This command changes the current y spacing and moves down by a.

 16. Opcodes 166-170: *zi (0 <= i <= 4); a[i]*

    The z0 command sets v:=v+z; i.e., moves down z units. The z commands are like the y commands except that they involve z instead of y. The other z commands set z:=a and v:=v+a. The value of a is a signed quantity in two's complement notation. This command changes the current z spacing and moves down by a.

 17. Opcodes 171-234: *fnt_num_i (0 <= i <= 63)*

    Set f:=i. Font i must previously have been defined by a fnt_def instruction, as explained below.

 18. Opcodes 235-238: *fnti (1 <= i <= 4); k[i]*

    Set f:=k. TeX82 uses the fnt1 command for font numbers in the range *64 < =k < 256*. TeX82 never generates the fnt2 command, but large font numbers may prove useful for specifications of color or texture, or they may be used for special fonts that have fixed numbers in some external coding scheme.

 19. Opcodes 239-242: *xxxi (1 <= i <= 4); k[i], x[k]*

    This command is undefined in general; it functions as a k+i+1$-byte nop unless special DVI-reading programs are being used. TeX82 generates xxx1 when a short enough \special appears, setting k to the number of bytes being sent. It is recommended that x be a string having the form of a keyword followed by possible parameters relevant to that keyword.

 20. Opcodes 243-246: *fnt_defi (1 <= i <= 4); k[i], c[4], s[4], d[4], a[1], l[1], n[a+l]*

    The four-byte value c is the check sum that TeX (or whatever program generated the DVI file) found in the TFM file for this font; c should match the check sum of the font found by programs that read this DVI file.

    Parameter s contains a fixed-point scale factor that is applied to the character widths in font k; font dimensions in TFM files and other font files are relative to this quantity, which is always positive and less than 2^27. It is given in the same units as the other dimensions of the DVI file. Parameter d is similar to s; it is the "design size," and (like s) it is given in DVI units. Thus, font k is to be used at mag s / 1000 d times its normal size.

    The remaining part of a font definition gives the external name of the font, which is an ASCII string of length a+l. The number a is the length of the "area" or directory, and l is the length of the font name itself; the standard local system font area is supposed to be used when a=0. The n field contains the area in its first a bytes.

    Font definitions must appear before the first use of a particular font number. Once font k is defined, it must not be defined again; however, we shall see below that font definitions appear in the postamble as well as in the pages, so in this sense each font number is defined exactly twice, if at all. Like nop commands, font definitions can appear before the first bop, or between an eop and a bop.

 21. Opcodes 247: pre; i[1], num[4], den[4], mag[4], k[1], x[k]

    The preamble contains basic information about the file as a whole and must come at the very beginning of the file. The i byte identifies DVI format; currently this byte is always set to 2. (The value i=3 is currently used for an extended format that allows a mixture of right-to-left and left-to-right typesetting. Some day we will set i=4, when DVI format makes another incompatible change - perhaps in the year 2048.)

    The next two parameters, num and den, are positive integers that define the units of measurement; they are the numerator and denominator of a fraction by which all dimensions in the DVI file could be multiplied in order to get lengths in units of 10^(-7) meters. (For example, there are exactly 7227 TeX points in 254 centimeters, and TeX82 works with scaled points where there are 2^16 sp in a point, so TeX82 sets num=25400000 and den=7227 2^16=473628672.

    The mag parameter is what TeX82 calls \mag, i.e., 1000 times the desired magnification. The actual fraction by which dimensions are multiplied is therefore m n /1000 d. Note that if a TeX source document does not call for any true dimensions, and if you change it only by specifying a different \mag setting, the DVI file that TeX creates will be completely unchanged except for the value of mag in the preamble and postamble. (Fancy DVI-reading programs allow users to override the mag setting when a DVI file is being printed.)

    Finally, k and x allow the DVI writer to include a comment, which is not interpreted further. The length of comment x is k, where *0 < = k < 256*.

 22. Opcodes 248: *post; p[4], num[4], den[4], mag[4], l[4], u[4], s[2], t[2]; < font definitions >*

    The last page in a DVI file is followed by post; this command introduces the postamble, which summarizes important facts that TeX has accumulated about the file, making it possible to print subsets of the data with reasonable efficiency. The parameter p is a pointer to the final bop in the file. The next three parameters, num, den, and mag, are duplicates of the quantities that appeared in the preamble.

    Parameters l and u give respectively the height-plus-depth of the tallest page and the width of the widest page, in the same units as other dimensions of the file. These numbers might be used by a DVI-reading program to position individual "pages" on large sheets of film or paper; however, the standard convention for output on normal size paper is to position each page so that the upper left-hand corner is exactly one inch from the left and the top. Experience has shown that it is unwise to design DVI-to-printer software that attempts cleverly to center the output; a fixed position of the upper left corner is easiest for users to understand and to work with. Therefore l and u are often ignored.

    Parameter s is the maximum stack depth (i.e., the largest excess of push commands over pop commands) needed to process this file. Then comes t, the total number of pages (bop commands) present.

    The postamble continues with font definitions, which are any number of fnt_def commands as described above, possibly interspersed with nop commands. Each font number that is used in the DVI file must be defined exactly twice: Once before it is first selected by a fnt command, and once in the postamble.

 23. Opcodes 249: post_post; q[4], i[1]; 223's

    The last part of the postamble, following the post_post byte that signifies the end of the font definitions, contains q a pointer to the post command that started the postamble. An identification byte, i, comes next; this currently equals 2, as in the preamble.

    The i byte is followed by four or more bytes that are all equal to the decimal number 223 (i.e., 337 in octal). TeX puts out four to seven of these trailing bytes, until the total length of the file is a multiple of four bytes, since this works out best on machines that pack four bytes per word; but any number of 223's is allowed, as long as there are at least four of them. In effect, 223 is a sort of signature that is added at the very end.

    This curious way to finish off a DVI file makes it feasible for DVI-reading programs to find the postamble first, on most computers, even though TeX wants to write the postamble last. Most operating systems permit random access to individual words or bytes of a file, so the DVI reader can start at the end and skip backwards over the 223's until finding the identification byte. Then it can back up four bytes, read q, and move to byte q of the file. This byte should, of course, contain the value 248 (post); now the postamble can be read, so the DVI reader discovers all the information needed for typesetting the pages. Note that it is also possible to skip through the DVI file at reasonably high speed to locate a particular page, if that proves desirable. This saves a lot of time, since DVI files used in production jobs tend to be large. 

[archived_spec]: https://web.archive.org/web/20070403030353/http://www.math.umd.edu/~asnowden/comp-cont/dvi.html#w
