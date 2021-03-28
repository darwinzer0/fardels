import React, { Component } from 'react';
import { connect } from 'react-redux';

import Box from '@material-ui/core/Box';
import Grid from '@material-ui/core/Grid';
import Paper from '@material-ui/core/Paper';
import Typography from '@material-ui/core/Typography';
import List from '@material-ui/core/List';
import ListItem from '@material-ui/core/ListItem';
import LockTwoToneIcon from '@material-ui/icons/LockTwoTone';
import VisibilityTwoToneIcon from '@material-ui/icons/VisibilityTwoTone';
import { withStyles } from '@material-ui/core/styles';

const useStyles = theme => ({
  button: {
    margin: theme.spacing(1),
  },
  root: {
    display: 'flex',
    flexWrap: 'wrap',
  },
  headerBox: {
    width: '100%',
    padding: theme.spacing(2),
  },
  paper: {
    padding: theme.spacing(2),
    margin: 'auto',
    width: '100%',
  },
  img: {
    margin: 'auto',
    display: 'block',
    maxWidth: 60,
    maxHeight: 60,
  },
});

class UnpackedFardels extends Component {

  render() {
  	const { classes, viewedFardels, viewedHandle } = this.props;

    return (
      <div className={classes.root}>
        <Box className={classes.headerBox}>
          <Typography variant="h6">{"@"+viewedHandle}</Typography>
        </Box>
        <List>
  	      {viewedFardels.map((fardel, index) => (
  	        <ListItem key={index}>
            <Paper className={classes.paper}>
              <Grid container spacing={2}>
                <Grid item>
                  { fardel.img &&
                    <img className={classes.img} alt="thumbnail" src={fardel.img} />
                  }
                </Grid>
                <Grid item xs={12} sm container>
                  <Grid item xs container direction="column" spacing={2}>
                    <Grid item xs>
                      <Typography gutterBottom>
                        {fardel.message}
                      </Typography>
                      <Typography variant="body2">
                        {Date(fardel.timestamp).toLocaleString()}
                      </Typography>
                    </Grid>
                    <Grid item>
                      <Typography variant="body2">
                        { fardel.locked
                          ? <LockTwoToneIcon />
                          : <VisibilityTwoToneIcon />
                        }
                      </Typography>
                    </Grid>
                  </Grid>
                  <Grid>
                    <Typography variant="subtitle1">SCRT 1.50</Typography>
                  </Grid>
                </Grid>
              </Grid>
            </Paper>
            </ListItem>
          ))}
        </List>
      </div>
    );
  }
}

const mapStateToProps = (state) => {
  return {
    keplrEnabled: state.keplrEnabled,
    viewedHandle: state.viewedHandle,
    viewedFardels: state.viewedFardels,
  }
};

const mapDispatchToProps = (dispatch) => {
  return ({

  });
}

export default connect(mapStateToProps, mapDispatchToProps)(withStyles(useStyles)(UnpackedFardels));