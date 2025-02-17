```
	!do homework by friday
	!do homework by 3pm
	!do homework by next friday
	!do homework by the 3rd
	!do homework by 2024-03-25
	!do homework by 03-25
	!do homework by march 3rd
	!do shit by friday
	!do shit by next friday
	!do shit by feb 2nd
	!do shit by 2025-02-02
	!do shit by 02-02

	# casing
	[date]-[date]-[None] // date
	[date]-[date]-[date] // date
	the [date] //date
	[day of week] //date
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

