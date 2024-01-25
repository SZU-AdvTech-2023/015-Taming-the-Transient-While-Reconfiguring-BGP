# Chameleon: Taming the transient while reconfiguring BGP
# Copyright (C) 2023 Tibor Schneider <sctibor@ethz.ch>
#
# This program is free software; you can redistribute it and/or modify
# it under the terms of the GNU General Public License as published by
# the Free Software Foundation; either version 2 of the License, or
# (at your option) any later version.
#
# This program is distributed in the hope that it will be useful,
# but WITHOUT ANY WARRANTY; without even the implied warranty of
# MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
# GNU General Public License for more details.
#
# You should have received a copy of the GNU General Public License along
# with this program; if not, write to the Free Software Foundation, Inc.,
# 51 Franklin Street, Fifth Floor, Boston, MA 02110-1301 USA.

import os
import pandas as pd
import plotly.express as px

from utils import select_measurement

if __name__ == "__main__":
    path = select_measurement(contains="throughput_per_egress.csv")
    file = os.path.join(path, "throughput_per_egress.csv")
    plot_file = os.path.join(path, "plot_testbed.html")

    df = pd.read_csv(file, sep=",")
    keys = [k for k in df.keys() if k not in {"t", "Sum", "violations"}]

    keys = ["Sum"] + ["violations"] + keys

    fig = px.line(df, x="t", y=keys)
    fig.write_html(plot_file)
    print(f"Written plot to {plot_file}")
