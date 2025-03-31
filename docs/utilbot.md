```
	!do homework by friday (yes yes)
	!do homework by 3pm (yes yes)
	!do homework by next friday (yes yes)
	!do homework by the 3rd (yes)
	!do homework by 2024-03-25 (yes)
	!do homework by 03-25 (yes)
	!do homework by march 3rd (yes)
	!do homework by march 3 

	the
	this
	next
	[month]

	# casing
	[date]-[date]-[None] // date
	[date]-[date]-[date] // date
	the [date] //date
	[day of week] //dates
	next [day of week] //date
	[number][am/pm/rd/nd/th] // hours
	[month (text)] [date][rd/nd/th]

	case whitespace:
	- one space:
		- the [date] // parse current 
		- next [day of week]
		- [month] [date]
	- no space:
		- [date]
		- [time]

	!do laundry in 3 minutes
	!do laundry in three minutes
	!do laundry in four days
	!do laundry in years
	!do bladee vinyl in 3 days
	!do chore in 3 hours

	# casing 
	(int or word to number)
	minutes / hours / days / weeks / months / years
```

bugs:

